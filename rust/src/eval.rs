use types::{Ast, Primitive};
use env::{Env, add, sub, mul, div};

pub fn eval(ast: &Ast, env: &mut Env) -> Option<Ast> {
    match ast {
        &Ast::Nil => Some(ast.clone()),
        &Ast::True => Some(ast.clone()),
        &Ast::False => Some(ast.clone()),
        &Ast::Number(_) => Some(ast.clone()),
        &Ast::Symbol(ref s) => env.get(s),
        &Ast::String(_) => Some(ast.clone()),
        &Ast::List(ref ls) => eval_application(ls.to_vec(), env),
        &Ast::Operator(_) => unreachable!(),
    }
}

fn eval_application(app: Vec<Ast>, env: &mut Env) -> Option<Ast> {
    let pair = app.split_first();

    if pair.is_none() {
        return Some(Ast::List(vec![]));
    }

    pair.and_then(|(op, ops)| {
            eval(op, env).and_then(|op| {
                match op {
                    Ast::Operator(op) => {
                        // grab primitive
                        Some((op, ops))
                    }
                    _ => unreachable!(),
                }
            })
        })
        .and_then(|(op, ops)| {
            let ops = ops.to_vec();
            match op {
                Primitive::Define => apply_define(env, ops),
                Primitive::Let => apply_let(env, ops),
                Primitive::Add => {
                    let ops = ops.into_iter()
                        .map(|op| eval(&op, env).unwrap())
                        .collect::<Vec<_>>();
                    apply_fn(add, ops)
                }
                Primitive::Subtract => {
                    let ops = ops.into_iter()
                        .map(|op| eval(&op, env).unwrap())
                        .collect::<Vec<_>>();
                    apply_fn(sub, ops)
                }
                Primitive::Multiply => {
                    let ops = ops.into_iter()
                        .map(|op| eval(&op, env).unwrap())
                        .collect::<Vec<_>>();
                    apply_fn(mul, ops)
                }
                Primitive::Divide => {
                    let ops = ops.into_iter()
                        .map(|op| eval(&op, env).unwrap())
                        .collect::<Vec<_>>();
                    apply_fn(div, ops)
                }
            }
        })
}

fn apply_define(env: &mut Env, args: Vec<Ast>) -> Option<Ast> {
    args.split_first()
        .and_then(|(key, unevals)| {
            match unevals.first() {
                Some(val) => Some((key, eval(val, env))),
                None => None,
            }
        })
        .and_then(|(key, val)| {
            match val {
                Some(v) => Some((key, v)),
                None => None,
            }
        })
        .and_then(|(key, val)| {
            match key {
                &Ast::Symbol(ref s) => Some((s.clone(), val)),
                _ => None,
            }
        })
        .and_then(|(key, val)| {
            env.set(key, val.clone());
            Some(val)
        })
}

fn apply_let(env: &mut Env, args: Vec<Ast>) -> Option<Ast> {
    build_let_env(env, args).and_then(|(ref mut env, ref body)| eval(body, env))
}

fn build_let_env<'a>(env: &'a Env, args: Vec<Ast>) -> Option<(Env<'a>, Ast)> {
    args.split_first()
        .and_then(|(bindings, bodies)| {
            match bodies.first() {
                Some(body) => Some((bindings, body)),
                None => None,
            }
        })
        .and_then(|(bindings, body)| {
            match bindings {
                &Ast::List(ref ls) => Some((ls.to_vec(), body)),
                _ => None,
            }
        })
        .and_then(|(bindings, body)| {
            if let Some(e) = let_env_from(bindings, env) {
                Some((e, body.clone()))
            } else {
                None
            }
        })
}

const PAIR_CHUNK_SIZE: usize = 2;

fn let_env_from<'a>(bindings: Vec<Ast>, env: &'a Env) -> Option<Env<'a>> {
    let mut env = Env::new(Some(Box::new(env)));
    for pair in bindings.chunks(PAIR_CHUNK_SIZE) {
        if pair.len() != PAIR_CHUNK_SIZE {
            break;
        }

        let key = match pair[0].clone() {
            Ast::Symbol(s) => s.clone(),
            _ => unreachable!(),
        };

        if let Some(val) = eval(&pair[1], &mut env) {
            env.set(key, val);
        } else {
            break;
        }
    }
    Some(env)
}

fn apply_fn<F>(f: F, args: Vec<Ast>) -> Option<Ast>
    where F: FnMut(Ast, Ast) -> Ast
{
    args.split_first()
        .and_then(|(first, rest)| {
            let result = rest.into_iter()
                .map(|a| a.clone())
                .fold(first.clone(), f);
            Some(result)
        })
        .or(None)
}
