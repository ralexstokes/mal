use types::Ast;
use ns;
use std::rc::Rc;
use std::cell::RefCell;
use env::{Env, empty_with, new};

pub fn eval(ast: &Ast, env: Env) -> Option<Ast> {
    match ast {
        &Ast::Nil => Some(ast.clone()),
        &Ast::Boolean(_) => Some(ast.clone()),
        &Ast::String(_) => Some(ast.clone()),
        &Ast::Number(_) => Some(ast.clone()),
        &Ast::Symbol(ref s) => env.borrow().get(s),
        // &Ast::If { predicate: ref p, consequent: ref c, alternative: ref a } => {
        //     match *a {
        //         Some(ref alt) => eval_if(*p.clone(), *c.clone(), Some(*alt.clone()), env),
        //         None => eval_if(*p.clone(), *c.clone(), None, env),
        //     }
        // }
        // &Ast::Do(ref seq) => eval_do(seq.to_vec(), env),
        &Ast::Lambda { .. } => Some(ast.clone()),
        &Ast::Fn(_) => Some(ast.clone()),
        // &Ast::Define { name: ref n, val: ref v } => eval_define(n.clone(), *v.clone(), env),
        // &Ast::Let { ref bindings, ref body } => eval_let(bindings.to_vec(), *body.clone(), env),
        &Ast::List(ref seq) => eval_combination(seq.to_vec(), env),
    }
}

fn eval_do(seq: Vec<Ast>, env: Env) -> Option<Ast> {
    seq.iter()
        .map(|s| eval(&s, env.clone()))
        .last()
        .unwrap_or(None)
}

fn eval_if(predicate: Ast, consequent: Ast, alternative: Option<Ast>, env: Env) -> Option<Ast> {
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
