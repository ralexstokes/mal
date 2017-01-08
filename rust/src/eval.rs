use types::Ast;
use ns;
use env::{Env, empty_from, new, root};

pub fn eval(ast: &Ast, env: Env) -> Option<Ast> {
    macroexpand(ast, env.clone()).and_then(|ast| {
        match ast {
            Ast::Symbol(ref s) => env.borrow().get(s),
            Ast::List(ref seq) => eval_list(seq.to_vec(), env),
            _ => ast.clone().into(), // self-evaluating
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

fn eval_list(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    if seq.is_empty() {
        return Some(Ast::List(seq));
    }

    seq.split_first().and_then(|(operator, operands)| {
        match *operator {
            Ast::Symbol(ref s) => {
                match s.as_str() {
                    IF_FORM => eval_if(operands.to_vec(), env),
                    SEQUENCE_FORM => eval_sequence(operands.to_vec(), env),
                    DEFINE_FORM => eval_define(operands.to_vec(), env),
                    LET_FORM => eval_let(operands.to_vec(), env),
                    LAMBDA_FORM => eval_lambda(operands.to_vec(), env),
                    EVAL_FORM => eval_eval(eval_ops(operands.to_vec(), env.clone()), env),
                    ENV_FORM => eval_env(env),
                    QUOTE_FORM => eval_quote(operands.to_vec()),
                    QUASIQUOTE_FORM => eval_quasiquote(operands.to_vec(), env),
                    MACRO_FORM => eval_macro(operands.to_vec(), env),
                    MACROEXPAND_FORM => eval_macroexpand(operands.to_vec(), env),
                    _ => apply(operator, eval_ops(operands.to_vec(), env.clone()), env),
                }
            }
            _ => apply(operator, eval_ops(operands.to_vec(), env.clone()), env),
        }
    })
}

fn eval_ops(operands: Vec<Ast>, env: Env) -> Vec<Ast> {
    operands.iter()
        .map(|operand| eval(operand, env.clone()))
        .filter(|operand| operand.is_some())
        .map(|operand| operand.unwrap())
        .collect::<Vec<_>>()
}

fn apply(operator: &Ast, evops: Vec<Ast>, env: Env) -> Option<Ast> {
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
}

fn eval_sequence(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    seq.iter()
        .map(|s| eval(&s, env.clone()))
        .last()
        .unwrap_or(None)
}

fn eval_if(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    if seq.len() < 2 {
        return None;
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
                    Some(Ast::Nil)
                }
            }
            _ => eval(&consequent, env),
        }
    })
}

fn eval_define(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    if seq.len() < 2 {
        return None;
    }

    let n = match seq[0] {
        Ast::Symbol(ref s) => s.clone(),
        _ => unreachable!(),
    };
    let ref val = seq[1];

    eval(val, env.clone()).and_then(|val| {
        env.borrow_mut().set(n, val.clone());
        Some(val)
    })
}

fn eval_let(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    seq.split_first()
        .and_then(|(bindings, body)| {
            if let Ast::List(ref seq) = *bindings {
                let body = body.to_vec();
                build_let_env(seq.to_vec(), env).and_then(|env| eval_sequence(body, env))
            } else {
                None
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

        if let Some(val) = eval(&pair[1], env.clone()) {
            env.borrow_mut().set(key, val);
        }
    }
    Some(env)
}

fn eval_lambda(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    if seq.len() < 2 {
        return None;
    }

    let params = match seq[0] {
        Ast::List(ref params) => params.to_vec(),
        _ => unreachable!(),
    };

    let body = seq[1..].to_vec();

    Ast::Lambda {
            params: params,
            body: body,
            env: env,
            is_macro: false,
        }
        .into()
}

// guest eval
fn eval_eval(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    // grab reference to root env in case we are
    // `eval`ing inside a temporary env (e.g. lambda, let)
    let root_env = root(&env);

    seq.first()
        .and_then(|arg| eval(arg, root_env.clone()))
}


// for debugging
fn eval_env(env: Env) -> Option<Ast> {
    env.borrow().inspect();
    Ast::Nil.into()
}


fn eval_quote(seq: Vec<Ast>) -> Option<Ast> {
    seq.first()
        .and_then(|quoted| quoted.clone().into())
}

fn eval_quasiquote(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    seq.first()
        .and_then(|arg| eval_quasiquote_for(arg, env.clone()))
        .and_then(|ast| eval(&ast, env))
}

fn eval_quasiquote_for(arg: &Ast, env: Env) -> Option<Ast> {
    let arg_elems = match *arg {
        Ast::List(ref seq) if !seq.is_empty() => seq.to_vec(),
        _ => {
            let mut result: Vec<Ast> = vec![];
            result.push(Ast::Symbol("quote".to_string()));
            result.push(arg.clone());
            return Ast::List(result).into();
        }
    };

    let first = &arg_elems[0];
    match *first {
        Ast::Symbol(ref s) if s == "unquote" => {
            if arg_elems.len() >= 2 {
                arg_elems[1].clone().into()
            } else {
                None
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

                                    eval_quasiquote_for(&next, env.clone()).and_then(|vals| {
                                        result.push(vals);
                                        Ast::List(result).into()
                                    })
                                })
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
                                    Ast::List(result).into()
                                })
                            })
                        }
                    }
                })
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
                    Ast::List(result).into()
                })
            })
        }
    }
}


fn eval_macro(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    if seq.len() < 2 {
        return None;
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
                Some(new_f)
            }
            _ => None,
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

fn macroexpand(ast: &Ast, env: Env) -> Option<Ast> {
    let mut result = ast.clone();
    while is_macro_call(&result, env.clone()) {
        let expansion = match result {
            Ast::List(ref seq) => {
                // using invariants of is_macro_call to skip some checks here
                match seq[0] {
                    Ast::Symbol(ref s) => {
                        let ast = match env.borrow().get(s) {
                            Some(ast) => ast.clone().into(),
                            None => None,
                        };
                        if let Some(ast) = ast {
                            apply(&ast, eval_ops(seq[1..].to_vec(), env.clone()), env.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        if let Some(val) = expansion {
            result = val;
        } else {
            return None;
        }
    }
    result.into()
}

fn eval_macroexpand(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    seq.first().and_then(|ast| macroexpand(ast, env))
}
