use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::result::Result;
use printer;
use env::Env;
use error::EvaluationError;

pub type LispValue = Rc<LispType>;

#[derive(Debug,Clone)]
pub enum LispType {
    Nil,
    Boolean(bool),
    String(String),
    Keyword(String),
    Number(i64),
    Symbol(String),
    Lambda {
        params: Vec<LispValue>,
        body: Vec<LispValue>,
        env: Env,
        is_macro: bool,
    },
    Fn(HostFn),
    List(Vec<LispValue>),
    Atom(RefCell<LispValue>),
}

pub type Seq = Vec<LispValue>;

pub type EvaluationResult = Result<LispValue, EvaluationError>;

pub type HostFn = fn(Seq) -> EvaluationResult;

fn value_of(t: LispType) -> LispValue {
    Rc::new(t)
}

pub fn new_nil() -> LispValue {
    value_of(LispType::Nil)
}

pub fn new_boolean(b: bool) -> LispValue {
    value_of(LispType::Boolean(b))
}

pub fn new_string(s: &str) -> LispValue {
    value_of(LispType::String(s.to_string()))
}

pub fn new_number(n: i64) -> LispValue {
    value_of(LispType::Number(n))
}

pub fn new_symbol(s: &str) -> LispValue {
    value_of(LispType::Symbol(s.to_string()))
}

pub fn new_lambda(params: Seq, body: Seq, env: Env, is_macro: bool) -> LispValue {
    value_of(LispType::Lambda {
        params: params,
        body: body,
        env: env,
        is_macro: is_macro,
    })
}

pub fn new_fn(f: HostFn) -> LispValue {
    value_of(LispType::Fn(f))
}

pub fn new_list(s: Seq) -> LispValue {
    value_of(LispType::List(s))
}

pub fn new_atom(atom: LispValue) -> LispValue {
    value_of(LispType::Atom(RefCell::new(atom)))
}

pub fn new_keyword(s: &str) -> LispValue {
    value_of(LispType::Keyword(s.to_string()))
}

impl fmt::Display for LispType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO need an RC just to satisfy the type here?
        write!(f, "{}", printer::print(value_of(self.clone())))
    }
}

impl PartialEq for LispType {
    fn eq(&self, other: &LispType) -> bool {
        use types::LispType::*;
        match (self, other) {
            (&Nil, &Nil) => true,
            (&Boolean(x), &Boolean(y)) => x == y,
            (&String(ref s), &String(ref t)) => s == t,
            (&Number(x), &Number(y)) => x == y,
            (&Keyword(ref s), &Keyword(ref t)) => s == t,
            (&Symbol(ref s), &Symbol(ref t)) => s == t,
            (&Lambda { .. }, &Lambda { .. }) => false,
            (&Fn(_), &Fn(_)) => false,
            (&List(ref xs), &List(ref ys)) => xs == ys,
            _ => false,
        }
    }
}
