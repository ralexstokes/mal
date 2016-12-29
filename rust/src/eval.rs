use types::Ast;
use ns;
use std::rc::Rc;
use std::cell::RefCell;
use env::Env;

pub fn eval(ast: &Ast, env: Rc<RefCell<Env>>) -> Option<Ast> {
    match ast {
        &Ast::Nil => Some(ast.clone()),
        &Ast::Boolean(_) => Some(ast.clone()),
        &Ast::String(_) => Some(ast.clone()),
        &Ast::Number(_) => Some(ast.clone()),
        &Ast::Symbol(ref s) => env.borrow().get(s),
        &Ast::If { predicate: ref p, consequent: ref c, alternative: ref a } => {
            match *a {
                Some(ref alt) => eval_if(*p.clone(), *c.clone(), Some(*alt.clone()), env),
                None => eval_if(*p.clone(), *c.clone(), None, env),
            }
        }
        &Ast::Do(ref seq) => eval_do(seq.to_vec(), env),
        &Ast::Lambda { .. } => Some(ast.clone()),
        &Ast::Fn(_) => Some(ast.clone()),
        &Ast::Define { name: ref n, val: ref v } => eval_define(n.clone(), *v.clone(), env),
        &Ast::Let { ref bindings, ref body } => eval_let(bindings.to_vec(), *body.clone(), env),
        &Ast::Combination(ref seq) => eval_combination(seq.to_vec(), env),
    }
}

fn eval_do(seq: Vec<Ast>, env: Rc<RefCell<Env>>) -> Option<Ast> {
    seq.iter()
        .map(|s| eval(&s, env.clone()))
        .last()
        .unwrap_or(None)
}

fn eval_if(predicate: Ast,
           consequent: Ast,
           alternative: Option<Ast>,
           env: Rc<RefCell<Env>>)
           -> Option<Ast> {
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

fn eval_lambda(params: Vec<Ast>,
               exprs: Box<Ast>,
               args: Vec<Ast>,
               env: Rc<RefCell<Env>>)
               -> Option<Ast> {
    let bindings = params.into_iter().zip(args.into_iter()).collect();
    let ns = ns::new(bindings);
    let new_env = Env::new(Some(env), ns);

    eval(&exprs, new_env)
}

fn eval_combination(app: Vec<Ast>, env: Rc<RefCell<Env>>) -> Option<Ast> {
    let pair = app.split_first();

    if pair.is_none() {
        return Some(Ast::Combination(vec![]));
    }

    pair.and_then(|(op, ops)| {
        eval(op, env.clone()).and_then(|op| {
            match op {
                Ast::Lambda { ref bindings, ref body, ref env } => {
                    if let Some(e) = env.clone() {
                        eval_lambda(bindings.to_vec(),
                                    body.clone(),
                                    eval_ops.to_vec(),
                                    e.clone())
                    } else {
                        None
                    }
                }
                Ast::Fn(f) => {
                    let ops = ops.iter()
                        .map(|ast| eval(ast, env.clone()).unwrap())
                        .collect::<Vec<_>>();
                    f(ops.to_vec())
                }
                _ => Some(op.clone()),
            }
        })
    })
}

fn eval_define(n: String, val: Ast, env: Rc<RefCell<Env>>) -> Option<Ast> {
    eval(&val, env.clone()).and_then(|val| {
        env.borrow_mut().set(n, val.clone());
        Some(val)
    })
}

fn eval_let(bindings: Vec<Ast>, body: Ast, env: Rc<RefCell<Env>>) -> Option<Ast> {
    build_let_env(bindings, env).and_then(|env| eval(&body, env))
}

const PAIR_CHUNK_SIZE: usize = 2;

fn build_let_env(bindings: Vec<Ast>, env: Rc<RefCell<Env>>) -> Option<Rc<RefCell<Env>>> {
    let env = Env::empty_with(Some(env));
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
