use std::fmt;
use printer;
use env::Env;

pub type HostFn = fn(Vec<Ast>) -> Option<Ast>;

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
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}",
               printer::print(self.clone()).unwrap_or("error".to_string()))
    }
}
