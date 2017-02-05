use types::{LispValue, LispType, Seq, EvaluationResult, new_symbol, new_list, new_nil, new_lambda,
            new_vector, Assoc, new_map_from_fn, new_string};
use std::result::Result;
use error::{error_message, EvaluationError};
use ns;
use env::{Env, empty_from, new, root};

pub fn eval(val: LispValue, env: Env) -> EvaluationResult {
    match *val {
        LispType::Symbol(ref s) => eval_symbol(s, env),
        LispType::List(ref seq) => {
            // TODO want to avoid re boxing this
            macroexpand(new_list(seq.clone()), env.clone()).and_then(|val| {
                if let LispType::List(ref seq) = *val {
                    return eval_list(seq.to_vec(), env);
                }

                eval(val, env)
            })
        }
        LispType::Vector(ref seq) => eval_vector(seq.to_vec(), env),
        LispType::Map(ref map) => eval_map(map, env),
        _ => eval_self_evaluating(val.clone()),
    }
}

fn eval_symbol(s: &str, env: Env) -> EvaluationResult {
    env.borrow().get(s)
}

fn eval_self_evaluating(val: LispValue) -> EvaluationResult {
    Ok(val)
}

const IF_FORM: &'static str = "if";
const SEQUENCE_FORM: &'static str = "do";
const DEFINE_FORM: &'static str = "def!";
const LET_FORM: &'static str = "let*";
const LAMBDA_FORM: &'static str = "fn*";
const EVAL_FORM: &'static str = "eval";
const ENV_FORM: &'static str = "env";
const QUOTE_FORM: &'static str = "quote";
const QUASIQUOTE_FORM: &'static str = "quasiquote";
const MACRO_FORM: &'static str = "defmacro!";
const MACROEXPAND_FORM: &'static str = "macroexpand";
const TRY_FORM: &'static str = "try*";
const CATCH_FORM: &'static str = "catch*";

fn eval_list(seq: Seq, env: Env) -> EvaluationResult {
    if seq.is_empty() {
        return Ok(new_list(seq));
    }

    seq.split_first()
        .ok_or(EvaluationError::Message("could not split list to eval".to_string()))
        .and_then(|(operator, operands)| {
            match **operator {
                LispType::Symbol(ref s) if s == IF_FORM => eval_if(operands.to_vec(), env),
                LispType::Symbol(ref s) if s == SEQUENCE_FORM => {
                    eval_sequence(operands.to_vec(), env)
                }
                LispType::Symbol(ref s) if s == DEFINE_FORM => eval_define(operands.to_vec(), env),
                LispType::Symbol(ref s) if s == LET_FORM => eval_let(operands.to_vec(), env),
                LispType::Symbol(ref s) if s == LAMBDA_FORM => eval_lambda(operands.to_vec(), env),
                LispType::Symbol(ref s) if s == EVAL_FORM => eval_eval(operands.to_vec(), env),
                LispType::Symbol(ref s) if s == ENV_FORM => eval_env(env),
                LispType::Symbol(ref s) if s == QUOTE_FORM => eval_quote(operands.to_vec()),
                LispType::Symbol(ref s) if s == QUASIQUOTE_FORM => {
                    eval_quasiquote(operands.to_vec(), env)
                }
                LispType::Symbol(ref s) if s == MACRO_FORM => eval_macro(operands.to_vec(), env),
                LispType::Symbol(ref s) if s == MACROEXPAND_FORM => {
                    eval_macroexpand(operands.to_vec(), env)
                }
                LispType::Symbol(ref s) if s == TRY_FORM => eval_try(operands.to_vec(), env),
                _ => apply(operator.clone(), operands.to_vec(), env),
            }
        })
}

fn eval_seq(operands: Seq, env: Env) -> Result<Seq, EvaluationError> {
    let mut result = vec![];
    for operand in operands.iter() {
        let evop = try!(eval(operand.clone(), env.clone()));
        result.push(evop);
    }
    Ok(result)
}

fn apply(operator: LispValue, operands: Seq, env: Env) -> EvaluationResult {
    eval_seq(operands, env.clone()).and_then(|evops| {
        eval(operator, env.clone()).and_then(|evop| {
            match *evop {
                LispType::Lambda { ref params, ref body, ref env, .. } => {
                    apply_lambda(params.clone(), body.clone(), env.clone(), evops.to_vec())
                }
                LispType::Fn(f) => f(evops.to_vec()),
                _ => unreachable!(),
            }
        })
    })
}

pub fn apply_lambda(params: Seq, body: Seq, env: Env, evops: Seq) -> EvaluationResult {
    let ns = ns::new_from(params, evops);
    let new_env = new(Some(env.clone()), ns);

    eval_sequence(body, new_env)
}

fn eval_sequence(seq: Seq, env: Env) -> EvaluationResult {
    let result = seq.iter()
        // TODO want to handle errors inside map here, not below
        .map(|s| eval(s.clone(), env.clone()))
        .last();
    if let Some(result) = result {
        result
    } else {
        Err(EvaluationError::Message("could not eval sequence".to_string()))
    }
}

fn eval_vector(seq: Seq, env: Env) -> EvaluationResult {
    eval_seq(seq, env).and_then(|seq| Ok(new_vector(seq)))
}

fn eval_map(map: &Assoc, env: Env) -> EvaluationResult {
    new_map_from_fn(map, |k, v| {
        let ev = try!(eval(v, env.clone()));
        Ok((k, ev))
    })
}

fn eval_if(seq: Seq, env: Env) -> EvaluationResult {
    if seq.len() < 2 {
        return Err(EvaluationError::Message("wrong arity".to_string()));
    }

    let predicate = &seq[0];
    let consequent = &seq[1];
    let alternative = if seq.len() >= 3 {
        Some(seq[2].clone())
    } else {
        None
    };

    eval(predicate.clone(), env.clone()).and_then(|p| {
        match *p {
            LispType::Nil |
            LispType::Boolean(false) => {
                if let Some(ref a) = alternative {
                    eval(a.clone(), env.clone())
                } else {
                    Ok(new_nil())
                }
            }
            _ => eval(consequent.clone(), env),
        }
    })
}

fn eval_define(seq: Seq, env: Env) -> EvaluationResult {
    if seq.len() < 2 {
        return Err(EvaluationError::Message("wrong arity".to_string()));
    }

    let name = match *seq[0] {
        LispType::Symbol(ref s) => s.clone(),
        _ => unreachable!(),
    };
    let val = &seq[1];

    eval(val.clone(), env.clone()).and_then(|val| {
        env.borrow_mut().set(name, val.clone());
        Ok(val)
    })
}

fn eval_let(seq: Seq, env: Env) -> EvaluationResult {
    seq.split_first()
        .ok_or(EvaluationError::Message("wrong arity".to_string()))
        .and_then(|(bindings, body)| {
            match **bindings {
                LispType::List(ref seq) |
                LispType::Vector(ref seq) => {
                    let body = body.to_vec();
                    build_let_env(seq.to_vec(), env)
                        .ok_or(EvaluationError::Message("could not build let env".to_string()))
                        .and_then(|env| eval_sequence(body, env))
                }
                _ => Err(EvaluationError::Message("wrong type!".to_string())),
            }
        })
}

const PAIR_CHUNK_SIZE: usize = 2;

fn build_let_env(bindings: Seq, env: Env) -> Option<Env> {
    let env = empty_from(env);
    for pair in bindings.chunks(PAIR_CHUNK_SIZE) {
        if pair.len() != PAIR_CHUNK_SIZE {
            break;
        }

        let key = match *pair[0].clone() {
            LispType::Symbol(ref s) => s.clone(),
            _ => unreachable!(),
        };

        if let Some(val) = eval(pair[1].clone(), env.clone()).ok() {
            env.borrow_mut().set(key, val);
        }
    }
    Some(env)
}

fn eval_lambda(seq: Seq, env: Env) -> EvaluationResult {
    if seq.len() < 2 {
        return Err(EvaluationError::Message("wrong arity".to_string()));
    }

    let params = match *seq[0] {
        LispType::List(ref params) |
        LispType::Vector(ref params) => params.to_vec(),
        _ => unreachable!(),
    };

    let body = seq[1..].to_vec();

    Ok(new_lambda(params, body, env, false))
}

// guest eval
fn eval_eval(seq: Seq, env: Env) -> EvaluationResult {
    eval_seq(seq, env.clone()).and_then(|seq| {
        // grab reference to root env in case we are
        // `eval`ing inside a temporary env (e.g. lambda, let)
        // otherwise, we will drop the new env frames after leaving
        // the temporary scope
        let root_env = root(&env);

        seq.first()
            .ok_or(EvaluationError::Message("wrong arity".to_string()))
            .and_then(|arg| eval(arg.clone(), root_env.clone()))
    })
}


// for debugging
fn eval_env(env: Env) -> EvaluationResult {
    env.borrow().inspect();
    Ok(new_nil())
}


fn eval_quote(seq: Seq) -> EvaluationResult {
    seq.first()
        .ok_or(EvaluationError::Message("wrong arity".to_string()))
        .and_then(|quoted| Ok(quoted.clone()))
}

fn eval_quasiquote(seq: Seq, env: Env) -> EvaluationResult {
    seq.first()
        .ok_or(EvaluationError::Message("wrong arity".to_string()))
        .and_then(|arg| eval_quasiquote_for(arg, env.clone()))
        .and_then(|val| eval(val, env))
}

fn eval_quasiquote_for(arg: &LispValue, env: Env) -> EvaluationResult {
    let arg_elems = match **arg {
        LispType::List(ref seq) if !seq.is_empty() => seq.to_vec(),
        LispType::Vector(ref seq) if !seq.is_empty() => seq.to_vec(),
        _ => {
            let mut result: Seq = vec![];
            result.push(new_symbol("quote"));
            result.push(arg.clone());
            return Ok(new_list(result));
        }
    };

    let first = &arg_elems[0];
    match **first {
        LispType::Symbol(ref s) if s == "unquote" => {
            if arg_elems.len() >= 2 {
                Ok(arg_elems[1].clone())
            } else {
                Err(EvaluationError::WrongArity(arg.clone()))
            }
        }
        LispType::List(ref seq) if !seq.is_empty() => {
            seq.split_first()
                .and_then(|(first, rest)| {
                    match **first {
                        LispType::Symbol(ref s) if s == "splice-unquote" => {
                            rest.first()
                                .and_then(|second| {
                                    let mut result: Seq = vec![];
                                    result.push(new_symbol("concat"));
                                    result.push(second.clone());

                                    let rest = arg_elems[1..].to_vec();
                                    let next = new_list(rest);

                                    eval_quasiquote_for(&next, env.clone())
                                        .and_then(|vals| {
                                            result.push(vals);
                                            Ok(new_list(result))
                                        })
                                        .ok()
                                })
                        }
                        _ => {
                            eval_quasiquote_for(&arg_elems[0], env.clone())
                                .and_then(|first| {
                                    let rest = arg_elems[1..].to_vec();
                                    let next = new_list(rest);
                                    eval_quasiquote_for(&next, env.clone()).and_then(|second| {
                                        let mut result: Seq = vec![];
                                        result.push(new_symbol("cons"));
                                        result.push(first.clone());
                                        result.push(second.clone());
                                        Ok(new_list(result))
                                    })
                                })
                                .ok() // TODO -- do not lose errors
                        }
                    }
                })
                .ok_or(EvaluationError::BadArguments(arg.clone()))
        }
        LispType::Vector(ref seq) if !seq.is_empty() => {
            // duplicate of above; TODO refactor
            seq.split_first()
                .and_then(|(first, rest)| {
                    match **first {
                        LispType::Symbol(ref s) if s == "splice-unquote" => {
                            rest.first()
                                .and_then(|second| {
                                    let mut result: Seq = vec![];
                                    result.push(new_symbol("concat"));
                                    result.push(second.clone());

                                    let rest = arg_elems[1..].to_vec();
                                    let next = new_list(rest);

                                    eval_quasiquote_for(&next, env.clone())
                                        .and_then(|vals| {
                                            result.push(vals);
                                            Ok(new_list(result))
                                        })
                                        .ok()
                                })
                        }
                        _ => {
                            eval_quasiquote_for(&arg_elems[0], env.clone())
                                .and_then(|first| {
                                    let rest = arg_elems[1..].to_vec();
                                    let next = new_list(rest);
                                    eval_quasiquote_for(&next, env.clone()).and_then(|second| {
                                        let mut result: Seq = vec![];
                                        result.push(new_symbol("cons"));
                                        result.push(first.clone());
                                        result.push(second.clone());
                                        Ok(new_vector(result))
                                    })
                                })
                                .ok() // TODO -- do not lose errors
                        }
                    }
                })
                .ok_or(EvaluationError::BadArguments(arg.clone()))
        }
        _ => {
            eval_quasiquote_for(&arg_elems[0], env.clone()).and_then(|first| {
                let rest = arg_elems[1..].to_vec();
                let next = new_list(rest);
                eval_quasiquote_for(&next, env.clone()).and_then(|second| {
                    let mut result: Seq = vec![];
                    result.push(new_symbol("cons"));
                    result.push(first.clone());
                    result.push(second.clone());
                    Ok(new_list(result))
                })
            })
        }
    }
}


fn eval_macro(seq: Seq, env: Env) -> EvaluationResult {
    if seq.len() < 2 {
        return Err(error_message("not enough arguments in call to defmacro!"));
    }

    let name = match *seq[0] {
        LispType::Symbol(ref s) => s.clone(),
        _ => unreachable!(),
    };
    let body = &seq[1];

    eval(body.clone(), env.clone()).and_then(|f| {
        match *f {
            LispType::Lambda { ref params, ref body, ref env, .. } => {
                let new_f = new_lambda(params.clone(), body.clone(), env.clone(), true);
                env.borrow_mut().set(name, new_f.clone());
                Ok(new_f)
            }
            _ => Err(EvaluationError::Message("Could not eval macro".to_string())),
        }
    })
}

fn is_macro_call(val: &LispValue, env: Env) -> bool {
    match **val {
        LispType::List(ref seq) => {
            if seq.is_empty() {
                return false;
            }

            match *seq[0] {
                LispType::Symbol(ref s) => {
                    match env.borrow().get(s) {
                        Ok(val) => {
                            match *val {
                                LispType::Lambda { is_macro, .. } => is_macro,
                                _ => false,
                            }
                        }
                        Err(_) => false,
                    }
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn macroexpand(val: LispValue, env: Env) -> EvaluationResult {
    let mut result = val.clone();
    while is_macro_call(&result, env.clone()) {
        let expansion = match *result {
            LispType::List(ref seq) => {
                // using invariants of is_macro_call to skip some checks here
                match *seq[0] {
                    LispType::Symbol(ref s) => {
                        env.borrow()
                            .get(s)
                            .and_then(|val| apply(val, seq[1..].to_vec(), env.clone()))
                    }
                    _ => Err(EvaluationError::BadArguments(val.clone())),
                }
            }
            _ => Err(EvaluationError::BadArguments(val.clone())),
        };
        match expansion {
            Ok(val) => result = val,
            Err(e) => return Err(e),
        }
    }
    Ok(result)
}

fn eval_macroexpand(seq: Seq, env: Env) -> EvaluationResult {
    seq.first()
        .ok_or(error_message("not enough args in call to macroexpand"))
        .and_then(|val| macroexpand(val.clone(), env))
}

fn eval_try(seq: Seq, env: Env) -> EvaluationResult {
    if seq.len() < 2 {
        return Err(EvaluationError::Message("wrong arity".to_string()));
    }

    let body = &seq[0];

    let result = eval(body.clone(), env.clone());
    // NOTE: currently just catches any error from the prior eval.
    // Need to handle Rust panics

    let handler = &seq[1];
    match result {
        Ok(val) => Ok(val),
        Err(EvaluationError::WrongArity(_)) => {
            eval_catch(handler.clone(), env.clone()) // will map catch form to lambda
                .and_then(|handler| eval_exception(handler, new_string("wrong arity"), env))
        }
        Err(EvaluationError::BadArguments(_)) => {
            eval_catch(handler.clone(), env.clone()) // will map catch form to lambda
                .and_then(|handler| eval_exception(handler, new_string("bad arguments to fn"), env))
        }
        Err(EvaluationError::MissingSymbol(ref sym)) => {
            eval_catch(handler.clone(), env.clone()) // will map catch form to lambda
                .and_then(|handler| eval_exception(handler, new_string(&format!("'{}' not found", sym)), env))
        }
        Err(EvaluationError::Message(ref msg)) => {
            eval_catch(handler.clone(), env.clone()) // will map catch form to lambda
                .and_then(|handler| eval_exception(handler, new_string(msg), env))
        }
        Err(EvaluationError::Exception(exn)) => {
            eval_catch(handler.clone(), env.clone()) // will map catch form to lambda
                .and_then(|handler| eval_exception(handler, exn, env))
        }
    }
}

// eval_catch builds a lambda from the given exception handler
// could add CATCH_FORM to eval block above, although this shortcuts that more general process
fn eval_catch(handler: LispValue, env: Env) -> EvaluationResult {
    let seq = match *handler {
        LispType::List(ref seq) if seq.len() >= 3 => seq.to_vec(),
        _ => return Err(EvaluationError::Message("wrong arity to catch handler".to_string())),
    };

    match *seq[0] {
        LispType::Symbol(ref s) if s != CATCH_FORM => {
            return Err(EvaluationError::Message("no catch handler supplied -- missing catch* ?"
                .to_string()));
        }
        _ => {}
    }

    let params = vec![seq[1].clone()];

    let body = vec![seq[2].clone()];

    Ok(new_lambda(params, body, env, false))
}

fn eval_exception(handler: LispValue, exception: LispValue, env: Env) -> EvaluationResult {
    match *handler {
        LispType::Lambda { ref params, ref body, .. } => {
            apply_lambda(params.clone(), body.clone(), env, vec![exception])
        }
        _ => unreachable!(),
    }
}
