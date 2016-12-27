use std::collections::HashMap;
use types::{Ast, Primitive};

pub struct Env {
    bindings: HashMap<String, Ast>,
    outer: Option<Box<Env>>,
}

impl Env {
    pub fn new(outer: Option<Box<Env>>) -> Env {
        Env {
            bindings: HashMap::new(),
            outer: outer,
        }
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

const DEFINE: &'static str = "def!";
const LET: &'static str = "let*";

pub fn add_default_bindings(env: &mut Env) {
    env.set("+".to_string(), Ast::Operator(Primitive::Add));
    env.set("-".to_string(), Ast::Operator(Primitive::Subtract));
    env.set("*".to_string(), Ast::Operator(Primitive::Multiply));
    env.set("/".to_string(), Ast::Operator(Primitive::Divide));
    env.set(DEFINE.to_string(), Ast::Operator(Primitive::Define));
    env.set(LET.to_string(), Ast::Operator(Primitive::Let));
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
