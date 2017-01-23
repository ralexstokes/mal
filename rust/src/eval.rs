use types::{Ast, EvaluationResult};
use std::result::Result;
use error::{error_message, EvaluationError};
use ns;
use env::{Env, empty_from, new, root};

pub fn eval(ast: &Ast, env: Env) -> EvaluationResult {
    macroexpand(ast, env.clone()).and_then(|ast| {
        match ast {
            Ast::Symbol(s) => {
                env.borrow()
                    .get(&s)
                    .ok_or(EvaluationError::MissingSymbol(s))
            }
            Ast::List(ref seq) => eval_list(seq.to_vec(), env),
            _ => Ok(ast.clone()), // self-evaluating
        }
    })
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


fn eval_list(seq: Vec<Ast>, env: Env) -> EvaluationResult {
    if seq.is_empty() {
        return Ok(Ast::List(seq));
    }

    seq.split_first()
        .ok_or(EvaluationError::Message("could not split list to eval".to_string()))
        .and_then(|(operator, operands)| {
            match *operator {
                Ast::Symbol(ref s) if s == IF_FORM => eval_if(operands.to_vec(), env),
                Ast::Symbol(ref s) if s == SEQUENCE_FORM => eval_sequence(operands.to_vec(), env),
                Ast::Symbol(ref s) if s == DEFINE_FORM => eval_define(operands.to_vec(), env),
                Ast::Symbol(ref s) if s == LET_FORM => eval_let(operands.to_vec(), env),
                Ast::Symbol(ref s) if s == LAMBDA_FORM => eval_lambda(operands.to_vec(), env),
                Ast::Symbol(ref s) if s == EVAL_FORM => eval_eval(operands.to_vec(), env),
                Ast::Symbol(ref s) if s == ENV_FORM => eval_env(env),
                Ast::Symbol(ref s) if s == QUOTE_FORM => eval_quote(operands.to_vec()),
                Ast::Symbol(ref s) if s == QUASIQUOTE_FORM => {
                    eval_quasiquote(operands.to_vec(), env)
                }
                Ast::Symbol(ref s) if s == MACRO_FORM => eval_macro(operands.to_vec(), env),
                Ast::Symbol(ref s) if s == MACROEXPAND_FORM => {
                    eval_macroexpand(operands.to_vec(), env)
                }
                Ast::Symbol(ref s) if s == TRY_FORM => eval_try(operands.to_vec(), env),
                _ => apply(operator, operands.to_vec(), env),
            }
        })
}

fn eval_ops(operands: Vec<Ast>, env: Env) -> Result<Vec<Ast>, EvaluationError> {
    let mut result = vec![];
    for operand in operands.iter() {
        let evop = try!(eval(operand, env.clone()));
        result.push(evop);
    }
    Ok(result)
}

fn apply(operator: &Ast, operands: Vec<Ast>, env: Env) -> EvaluationResult {
    eval_ops(operands, env.clone()).and_then(|evops| {
        eval(operator, env.clone()).and_then(|evop| {
            match evop {
                Ast::Lambda { params, body, env, .. } => {
                    let ns = ns::new_from(params, evops);
                    let new_env = new(Some(env.clone()), ns);

                    eval_sequence(body, new_env)
                }
                Ast::Fn(f) => f(evops.to_vec()),
                _ => unreachable!(),
            }
        })
    })
}

fn eval_sequence(seq: Vec<Ast>, env: Env) -> EvaluationResult {
    let result = seq.iter()
        // TODO want to handle errors inside map here, not below
        .map(|s| eval(&s, env.clone()))
        .last();
    if let Some(result) = result {
        result
    } else {
        Err(EvaluationError::Message("could not eval sequence".to_string()))
    }
}

fn eval_if(seq: Vec<Ast>, env: Env) -> EvaluationResult {
    if seq.len() < 2 {
        return Err(EvaluationError::Message("wrong arity".to_string()));
    }

    let ref predicate = seq[0];
    let ref consequent = seq[1];
    let alternative = if seq.len() >= 3 {
        Some(seq[2].clone())
    } else {
        None
    };

    eval(&predicate, env.clone()).and_then(|p| {
        match p {
            Ast::Nil |
            Ast::Boolean(false) => {
                if let Some(ref a) = alternative {
                    eval(a, env.clone())
                } else {
                    Ok(Ast::Nil)
                }
            }
            _ => eval(&consequent, env),
        }
    })
}

fn eval_define(seq: Vec<Ast>, env: Env) -> EvaluationResult {
    if seq.len() < 2 {
        return Err(EvaluationError::Message("wrong arity".to_string()));
    }

    let n = match seq[0] {
        Ast::Symbol(ref s) => s.clone(),
        _ => unreachable!(),
    };
    let ref val = seq[1];

    eval(val, env.clone()).and_then(|val| {
        env.borrow_mut().set(n, val.clone());
        Ok(val)
    })
}

fn eval_let(seq: Vec<Ast>, env: Env) -> EvaluationResult {
    seq.split_first()
        .ok_or(EvaluationError::Message("wrong arity".to_string()))
        .and_then(|(bindings, body)| {
            if let Ast::List(ref seq) = *bindings {
                let body = body.to_vec();
                build_let_env(seq.to_vec(), env)
                    .ok_or(EvaluationError::Message("could not build let env".to_string()))
                    .and_then(|env| eval_sequence(body, env))
            } else {
                Err(EvaluationError::Message("wrong type!".to_string()))
            }
        })
}

const PAIR_CHUNK_SIZE: usize = 2;

fn build_let_env(bindings: Vec<Ast>, env: Env) -> Option<Env> {
    let env = empty_from(env);
    for pair in bindings.chunks(PAIR_CHUNK_SIZE) {
        if pair.len() != PAIR_CHUNK_SIZE {
            break;
        }

        let key = match pair[0].clone() {
            Ast::Symbol(s) => s.clone(),
            _ => unreachable!(),
        };

        if let Some(val) = eval(&pair[1], env.clone()).ok() {
            env.borrow_mut().set(key, val);
        }
    }
    Some(env)
}

fn eval_lambda(seq: Vec<Ast>, env: Env) -> EvaluationResult {
    if seq.len() < 2 {
        return Err(EvaluationError::Message("wrong arity".to_string()));
    }

    let params = match seq[0] {
        Ast::List(ref params) => params.to_vec(),
        _ => unreachable!(),
    };

    let body = seq[1..].to_vec();

    Ok(Ast::Lambda {
        params: params,
        body: body,
        env: env,
        is_macro: false,
    })
}

// guest eval
fn eval_eval(seq: Vec<Ast>, env: Env) -> EvaluationResult {
    eval_ops(seq, env.clone()).and_then(|seq| {
        // grab reference to root env in case we are
        // `eval`ing inside a temporary env (e.g. lambda, let)
        let root_env = root(&env);

        seq.first()
            .ok_or(EvaluationError::Message("wrong arity".to_string()))
            .and_then(|arg| eval(arg, root_env.clone()))
    })
}


// for debugging
fn eval_env(env: Env) -> EvaluationResult {
    env.borrow().inspect();
    Ok(Ast::Nil)
}


fn eval_quote(seq: Vec<Ast>) -> EvaluationResult {
    seq.first()
        .ok_or(EvaluationError::Message("wrong arity".to_string()))
        .and_then(|quoted| Ok(quoted.clone()))
}

fn eval_quasiquote(seq: Vec<Ast>, env: Env) -> EvaluationResult {
    seq.first()
        .ok_or(EvaluationError::Message("wrong arity".to_string()))
        .and_then(|arg| eval_quasiquote_for(arg, env.clone()))
        .and_then(|ast| eval(&ast, env))
}

fn eval_quasiquote_for(arg: &Ast, env: Env) -> EvaluationResult {
    let arg_elems = match *arg {
        Ast::List(ref seq) if !seq.is_empty() => seq.to_vec(),
        _ => {
            let mut result: Vec<Ast> = vec![];
            result.push(Ast::Symbol("quote".to_string()));
            result.push(arg.clone());
            return Ok(Ast::List(result));
        }
    };

    let first = &arg_elems[0];
    match *first {
        Ast::Symbol(ref s) if s == "unquote" => {
            if arg_elems.len() >= 2 {
                Ok(arg_elems[1].clone())
            } else {
                Err(EvaluationError::WrongArity(arg.clone()))
            }
        }
        Ast::List(ref seq) if !seq.is_empty() => {
            seq.split_first()
                .and_then(|(first, rest)| {
                    match *first {
                        Ast::Symbol(ref s) if s == "splice-unquote" => {
                            rest.first()
                                .and_then(|second| {
                                    let mut result: Vec<Ast> = vec![];
                                    result.push(Ast::Symbol("concat".to_string()));
                                    result.push(second.clone());

                                    let rest = arg_elems[1..].to_vec();
                                    let next = Ast::List(rest);

                                    eval_quasiquote_for(&next, env.clone())
                                        .and_then(|vals| {
                                            result.push(vals);
                                            Ok(Ast::List(result))
                                        })
                                        .ok()
                                })
                        }
                        _ => {
                            eval_quasiquote_for(&arg_elems[0], env.clone())
                                .and_then(|first| {
                                    let rest = arg_elems[1..].to_vec();
                                    let next = Ast::List(rest);
                                    eval_quasiquote_for(&next, env.clone()).and_then(|second| {
                                        let mut result: Vec<Ast> = vec![];
                                        result.push(Ast::Symbol("cons".to_string()));
                                        result.push(first.clone());
                                        result.push(second.clone());
                                        Ok(Ast::List(result))
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
                let next = Ast::List(rest);
                eval_quasiquote_for(&next, env.clone()).and_then(|second| {
                    let mut result: Vec<Ast> = vec![];
                    result.push(Ast::Symbol("cons".to_string()));
                    result.push(first.clone());
                    result.push(second.clone());
                    Ok(Ast::List(result))
                })
            })
        }
    }
}


fn eval_macro(seq: Vec<Ast>, env: Env) -> EvaluationResult {
    if seq.len() < 2 {
        return Err(error_message("not enough arguments in call to defmacro!"));
    }

    let n = match seq[0] {
        Ast::Symbol(ref s) => s.clone(),
        _ => unreachable!(),
    };
    let ref val = seq[1];

    eval(val, env.clone()).and_then(|f| {
        match f {
            Ast::Lambda { params, body, env, .. } => {
                let new_f = Ast::Lambda {
                    params: params.clone(),
                    body: body.clone(),
                    env: env.clone(),
                    is_macro: true,
                };
                env.borrow_mut().set(n, new_f.clone());
                Ok(new_f)
            }
            _ => Err(EvaluationError::Message("Could not eval macro".to_string())),
        }
    })
}

fn is_macro_call(ast: &Ast, env: Env) -> bool {
    match *ast {
        Ast::List(ref seq) => {
            if seq.is_empty() {
                return false;
            }

            match seq[0] {
                Ast::Symbol(ref s) => {
                    match env.borrow().get(s) {
                        Some(ast) => {
                            match ast {
                                Ast::Lambda { is_macro, .. } => is_macro.into(),
                                _ => false.into(),
                            }
                        }
                        None => false,
                    }
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn macroexpand(ast: &Ast, env: Env) -> EvaluationResult {
    let mut result = ast.clone();
    while is_macro_call(&result, env.clone()) {
        let expansion = match result {
            Ast::List(ref seq) => {
                // using invariants of is_macro_call to skip some checks here
                match seq[0] {
                    Ast::Symbol(ref s) => {
                        env.borrow()
                            .get(s)
                            .ok_or(EvaluationError::Message("macroexpand: missing symbol"
                                .to_string()))
                            .and_then(|ast| apply(&ast, seq[1..].to_vec(), env.clone()))
                    }
                    _ => Err(EvaluationError::BadArguments(ast.clone())),
                }
            }
            _ => Err(EvaluationError::BadArguments(ast.clone())),
        };
        match expansion {
            Ok(val) => result = val,
            Err(e) => return Err(e),
        }
    }
    Ok(result)
}

fn eval_macroexpand(seq: Vec<Ast>, env: Env) -> EvaluationResult {
    seq.first()
        .ok_or(error_message("not enough args in call to macroexpand"))
        .and_then(|ast| macroexpand(ast, env))
}

fn eval_try(seq: Vec<Ast>, env: Env) -> EvaluationResult {
    if seq.len() < 2 {
        return Err(EvaluationError::Message("wrong arity".to_string()));
    }

    let body = &seq[0];
    let handler = &seq[1];

    let result = eval(body, env.clone());
    if let Err(EvaluationError::Exception(exn)) = result {
        eval_catch(handler, env.clone()) // will map catch form to lambda
            .and_then(|handler| eval_exception(handler, exn.clone(), env))
    } else {
        result
    }
}

// eval_catch builds a lambda from the given exception handler
// could add CATCH_FORM to eval block above, although this shortcuts that more general process
fn eval_catch(handler: &Ast, env: Env) -> EvaluationResult {
    let seq = match *handler {
        Ast::List(ref seq) if seq.len() >= 3 => seq.to_vec(),
        _ => return Err(EvaluationError::Message("wrong arity to catch handler".to_string())),
    };

    if seq[0] != Ast::Symbol(CATCH_FORM.to_string()) {
        return Err(EvaluationError::Message("no catch handler supplied -- missing catch* ?"
            .to_string()));
    }

    let params = vec![seq[1].clone()];

    let body = vec![seq[2].clone()];

    Ok(Ast::Lambda {
        params: params,
        body: body,
        env: env,
        is_macro: false,
    })
}

fn eval_exception(handler: Ast, exception: Ast, env: Env) -> EvaluationResult {
    let body = vec![handler, exception];
    let ast = Ast::List(body);
    eval(&ast, env)
}
