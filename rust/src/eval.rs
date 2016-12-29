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
        &Ast::Lambda{..} => Some(ast.clone()),
        &Ast::Fn(_) => Some(ast.clone()),
        &Ast::Define {
            name: ref n,
            val: ref v,
        } => eval_define(n.clone(), *v.clone(), env),
        &Ast::Let {
            bindings: ref bs,
            ref body,
        } => eval_let(bs.to_vec(), *body.clone(), env),
        &Ast::Combination(ref seq) => eval_combination(seq.to_vec(), env),
        &Ast::Operator(_) => unreachable!(),
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

fn eval_lambda(bindings: Vec<Ast>, exprs: Vec<Ast>, args: Vec<Ast>, env: Rc<RefCell<Env>>) -> Option<Ast> {
    None
    // let binds = bindings.iter()
    //     .map(|b| {
    //         match b {
    //             &Ast::Symbol(ref s) => s.as_str(),
    //             _ => unreachable!(),
    //         }
    //     })
    //     .collect::<Vec<_>>();
    // let body = |args| {
    //     let env = Env::new(Some(env), binds, exprs);
    //     eval(&args, env);
    // };
    // body(Ast::Nil)
    /*
    fn*: Return a new function closure. The body of that closure does the following:
    Create a new environment using env (closed over from outer scope) as the outer parameter, the first parameter (second list element of ast from the outer scope) as the binds parameter, and the parameters to the closure as the exprs parameter.

        Call EVAL on the second parameter (third list element of ast from outer scope), using the new environment. Use the result as the return value of the closure.

    // ( (fn* (a b) (+ b a)) 3 4)
    */
    // let mut new_env = env;
    // eval(&Ast::Nil, new_env)
}

fn eval_combination(app: Vec<Ast>, env: Rc<RefCell<Env>>) -> Option<Ast> {
    let pair = app.split_first();

    if pair.is_none() {
        return Some(Ast::Combination(vec![]));
    }

    pair.and_then(|(op, ops)| {
            eval(op, env.clone()).and_then(|op| {
                match op {
                    Ast::Lambda {
                        bindings: ref bs,
                        body: ref exprs,
                        env: ref env,
                    } => return eval_lambda(bs.to_vec(), exprs.to_vec(), ops.to_vec(), env.clone()),
                    Ast::Fn(f) => {
                        let ops = ops.iter()
                            .map(|ast| eval(ast, env.clone()).unwrap())
                            .collect::<Vec<_>>();
                        Some(f(ops.to_vec()))
                    }
                    _ => unreachable!(),
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
