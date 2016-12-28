use std::collections::HashMap;
use std::default::Default;
use types::Ast;

pub struct Env<'a> {
    bindings: HashMap<String, Ast>,
    outer: Option<Box<&'a Env<'a>>>,
}

impl<'a> Env<'a> {
    pub fn new<'b>(outer: Option<Box<&'b Env>>, binds: Vec<&str>, exprs: Vec<Ast>) -> Env<'b> {
        let mut e = Env {
            bindings: HashMap::new(),
            outer: outer,
        };
        for pair in binds.iter().zip(exprs.iter()) {
            let bind = pair.0;
            let expr = pair.1.clone();

            e.set(bind.to_string(), expr)
        }
        e
    }

    pub fn empty<'b>(outer: Option<Box<&'b Env>>) -> Env<'b> {
        Self::new(outer, vec![], vec![])
    }

    pub fn set(&mut self, key: String, val: Ast) {
        self.bindings.insert(key, val);
    }

    pub fn find(&self, key: &String) -> Option<&Env> {
        if self.bindings.contains_key(key) {
            return Some(self);
        } else {
            if let Some(ref env) = self.outer {
                env.find(key)
            } else {
                None
            }
        }
    }

    pub fn get(&self, key: &String) -> Option<Ast> {
        self.find(key)
            .and_then(|env| env.bindings.get(key))
            .map(|ast| ast.clone())
    }
}

impl<'a> Default for Env<'a> {
    fn default() -> Env<'a> {
        let binds = vec![
            "+",
            // "-",
            // "*",
            // "/",
        ];
        let exprs: Vec<Ast> = vec![
            // Primitive::Add,
            // Primitive::Subtract,
            // Primitive::Multiply,
            // Primitive::Divide,
            // Primitive::Define,
            // Primitive::Let,
        ];
        // .iter()
        // .map(|e| Ast::Operator(e.clone())).collect();

        Env::new(None, binds, exprs)
    }
}

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

pub fn add(a: Ast, b: Ast) -> Ast {
    let (a, b) = i64_from_ast(a, b);
    Ast::Number(a + b)
}

pub fn sub(a: Ast, b: Ast) -> Ast {
    let (a, b) = i64_from_ast(a, b);
    Ast::Number(a - b)
}

pub fn mul(a: Ast, b: Ast) -> Ast {
    let (a, b) = i64_from_ast(a, b);
    Ast::Number(a * b)
}

pub fn div(a: Ast, b: Ast) -> Ast {
    let (a, b) = i64_from_ast(a, b);
    Ast::Number(a / b)
}
