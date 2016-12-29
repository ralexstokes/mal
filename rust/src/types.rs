use std::fmt;
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
    },
    Fn(HostFn),
    List(Vec<Ast>),
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Ast::Nil => write!(f, "nil"),
            &Ast::Boolean(b) => write!(f, "{}", b),
            &Ast::String(ref s) => write!(f, "{}", s.clone()),
            &Ast::Number(n) => write!(f, "{}", n),
            &Ast::Symbol(ref s) => write!(f, "{}", s.clone()),
            &Ast::List(ref seq) => {
                let results = seq.into_iter()
                    .map(|node| format!("{}", node.clone()))
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "({})", results)
            }
            &Ast::Lambda { .. } => write!(f, "#<fn>"),
            &Ast::Fn(_) => write!(f, "#<host-fn>"),
        }
    }
}
