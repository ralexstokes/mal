use types::{Ast, Primitive};
use env::{Env, add, sub, mul, div};

pub fn eval(ast: &Ast, env: &mut Env) -> Option<Ast> {
    match ast {
        &Ast::Number(_) => Some(ast.clone()),
        &Ast::Symbol(ref s) => env.get(s),
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
        eval(op,env).and_then(|op| {
            match op {
                Ast::Operator(op) => { // grab primitive
                    Some((op, ops))
                },
                _ => unreachable!()
            }
        })
    }).and_then(|(op, ops)| {
        let ops = ops.to_vec();
        match op {
            Primitive::Define => {
                apply_define(env, ops)
            }
            Primitive::Let => {
                apply_let(env, ops)
            }
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
    args.split_first().and_then(|(key, unevals)| {
        match unevals.first() {
            Some(val) => Some((key, eval(val, env))),
            None => None
        }
    }).and_then(|(key, val)| {
        match val {
            Some(v) => Some((key,v)),
            None => None
        }
    }).and_then(|(key, val)| {
        match key {
            &Ast::Symbol(ref s) => {
                Some((s.clone(),val))
            },
            _ => None
        }
    }).and_then(|(key, val)| {
        env.set(key, val);
        Some(Ast::Symbol("ok".to_string()))
    })
}

fn apply_let(env: &Env, args: Vec<Ast>) -> Option<Ast> {
    /*
    symbol "let*": create a new environment using the current environment as the outer value and then use the first parameter as a list of new bindings in the "let*" environment. Take the second element of the binding list, call EVAL using the new "let*" environment as the evaluation environment, then call set on the "let*" environment using the first binding list element as the key and the evaluated second element as the value. This is repeated for each odd/even pair in the binding list. Note in particular, the bindings earlier in the list can be referred to by later bindings. Finally, the second parameter (third element) of the original let* form is evaluated using the new "let*" environment and the result is returned as the result of the let* (the new let environment is discarded upon completion).
    */
    None
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
