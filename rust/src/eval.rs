use types::Ast;
use ns;
use env::{Env, empty_from, new};

pub fn eval(ast: &Ast, env: Env) -> Option<Ast> {
    match ast {
        &Ast::Symbol(ref s) => env.borrow().get(s),
        &Ast::List(ref seq) => eval_list(seq.to_vec(), env),
        _ => ast.clone().into(), // self-evaluating
    }
}

const IF_FORM: &'static str = "if";
const DO_FORM: &'static str = "do";
const DEFINE_FORM: &'static str = "def!";
const LET_FORM: &'static str = "let*";
const LAMBDA_FORM: &'static str = "fn*";

fn eval_list(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    if seq.len() == 0 {
        return Some(Ast::List(seq));
    }

    seq.split_first()
        .and_then(|(operator, operands)| {
            match operator {
                &Ast::Symbol(ref s) => {
                    match s.as_str() {
                        IF_FORM => eval_if(operands.to_vec(), env),
                        DO_FORM => eval_do(operands.to_vec(), env),
                        DEFINE_FORM => eval_define(operands.to_vec(), env),
                        LET_FORM => eval_let(operands.to_vec(), env),
                        LAMBDA_FORM => eval_lambda(operands.to_vec(), env),
                        _ => apply(operator, operands.to_vec(), env),
                    }
                }
                _ => apply(operator, operands.to_vec(), env),
            }
        })
}

fn apply(operator: &Ast, operands: Vec<Ast>, env: Env) -> Option<Ast> {
    let evops = operands.iter()
        .map(|operand| eval(operand, env.clone()))
        .filter(|operand| operand.is_some())
        .map(|operand| operand.unwrap())
        .collect::<Vec<_>>();

    eval(operator, env).and_then(|evop| {
        match evop {
            Ast::Lambda { ref params, ref body, ref env } => {
                let bindings = params.into_iter()
                    .map(|p| {
                        match *p {
                            Ast::Symbol(ref s) => s.clone(),
                            _ => unreachable!(),
                        }
                    })
                    .zip(evops.into_iter())
                    .collect();
                let ns = ns::new(bindings);
                let new_env = new(Some(env.clone()), ns);

                eval(body, new_env)
            }
            Ast::Fn(f) => f(evops.to_vec()),
            _ => Some(evop.clone()),
        }
    })
}

fn eval_do(seq: Vec<Ast>, env: Env) -> Option<Ast> {
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
                let body = Ast::List(body.to_vec());
                build_let_env(seq.to_vec(), env).and_then(|env| eval(&body, env))
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

fn do_from(body: &[Ast]) -> Ast {
    let mut seq = vec![Ast::Symbol("do".to_string())];
    seq.append(&mut body.to_vec());
    Ast::List(seq)
}

fn eval_lambda(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    if seq.len() < 2 {
        return None;
    }

    let params = match seq[0] {
        Ast::List(ref params) => params.to_vec(),
        _ => unreachable!(),
    };

    let body = do_from(&seq[1..]);

    Ast::Lambda {
            params: params,
            body: Box::new(body),
            env: env,
        }
        .into()
}
