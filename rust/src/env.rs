use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use types::Ast;

#[derive(Debug)]
pub struct Env {
    bindings: HashMap<String, Ast>,
    outer: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new(outer: Option<Rc<RefCell<Env>>>,
               binds: Vec<&str>,
               exprs: Vec<Ast>)
               -> Rc<RefCell<Env>> {
        let mut e = Env {
            bindings: HashMap::new(),
            outer: outer,
        };
        for pair in binds.iter().zip(exprs.iter()) {
            let bind = pair.0;
            let expr = pair.1.clone();

            e.set(bind.to_string(), expr)
        }
        Rc::new(RefCell::new(e))
    }

    pub fn empty() -> Rc<RefCell<Env>> {
        Self::empty_with(None)
    }

    pub fn empty_with(outer: Option<Rc<RefCell<Env>>>) -> Rc<RefCell<Env>> {
        Self::new(outer, vec![], vec![])
    }

    pub fn set(&mut self, key: String, val: Ast) {
        self.bindings.insert(key, val);
    }

    // pub fn find(self, key: &String) -> Option<Box<Env>> {
    //     if self.bindings.contains_key(key) {
    //         Some(Box::new(self))
    //     } else {
    //         if let Some(env) = self.outer {
    //             env.find(key)
    //         } else {
    //             None
    //         }
    //     }
    // }

    pub fn get(&self, key: &String) -> Option<Ast> {
        self.bindings
            .get(key)
            .and_then(|val| Some(val.clone()))
            .or_else(|| {
                if let Some(ref env) = self.outer {
                    env.borrow().get(key)
                } else {
                    None
                }
            })
    }

    pub fn core() -> Rc<RefCell<Env>> {
        let binds = vec!["+", "-", "*", "/"];
        let exprs: Vec<Ast> = vec![Ast::Fn(add), Ast::Fn(sub), Ast::Fn(mul), Ast::Fn(div)];
        Self::new(None, binds, exprs)
    }
}

// impl Default for Env {
//     fn default() -> Env {
//         // .iter()
//         Env::new(None, binds, exprs)
//     }
// }

fn i64_from_ast(a: Ast, b: Ast) -> (i64, i64) {
    let aa = match a {
        Ast::Number(x) => x,
        _ => 0,
    };

    let bb = match b {
        Ast::Number(x) => x,
        _ => 0,
    };

    (aa, bb)
}

fn fold_first<F>(xs: Vec<Ast>, f: F) -> Ast
    where F: Fn(Ast, Ast) -> Ast
{
    xs.split_first()
        .and_then(|(first, rest)| {
            let result = rest.iter()
                .map(|a| a.clone())
                .fold(first.clone(), f);
            Some(result)
        })
        .unwrap_or(Ast::Nil)
}

pub fn add(xs: Vec<Ast>) -> Ast {
    fold_first(xs, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a + b)
    })
}

pub fn sub(xs: Vec<Ast>) -> Ast {
    fold_first(xs, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a - b)
    })
}

pub fn mul(xs: Vec<Ast>) -> Ast {
    fold_first(xs, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a * b)
    })
}

pub fn div(xs: Vec<Ast>) -> Ast {
    fold_first(xs, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a / b)
    })
}
