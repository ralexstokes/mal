use std::fmt;
use std::cmp::PartialEq;
use printer;
use env::Env;
use error::EvaluationError;

pub type EvaluationResult = ::std::result::Result<Ast, EvaluationError>;

pub type HostFn = fn(Vec<Ast>) -> EvaluationResult;

#[derive(Debug,Clone)]
pub enum Ast {
    Nil,
    Boolean(bool),
    String(String),
    Number(i64),
    Symbol(String),
    Lambda {
        params: Vec<Ast>,
        body: Vec<Ast>,
        env: Env,
        is_macro: bool,
    },
    Fn(HostFn),
    List(Vec<Ast>),
    Exception(Box<Ast>),
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", printer::print(self))
    }
}

impl PartialEq for Ast {
    fn eq(&self, other: &Ast) -> bool {
        use types::Ast::*;
        match (self.clone(), other.clone()) {
            (Nil, Nil) => true,
            (Boolean(x), Boolean(y)) if x == y => true,
            (String(ref s), String(ref t)) if s == t => true,
            (Number(x), Number(y)) if x == y => true,
            (Symbol(ref s), Symbol(ref t)) if s == t => true,
            (Lambda { .. }, Lambda { .. }) => false,
            (Fn(f), Fn(g)) if f == g => true,
            (List(xs), List(ys)) => xs == ys,
            _ => false,
        }
    }
}
