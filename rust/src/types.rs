use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::result::Result;
use std::collections::HashMap;
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
    Vector(Vec<LispValue>),
    Map(Assoc),
    Atom(RefCell<LispValue>),
}

#[derive(Debug,Clone)]
pub struct Assoc {
    pub bindings: HashMap<String, LispValue>,
}

impl Assoc {
    pub fn new() -> Assoc {
        Assoc { bindings: HashMap::new() }
    }

    pub fn from_seq(seq: Seq) -> Result<Assoc, EvaluationError> {
        if seq.len() % 2 != 0 {
            return Err(EvaluationError::Message("need an even number of elements to make a map"
                .to_string()));
        }

        let mut map = Assoc::new();

        for pair in seq.chunks(2) {
            let k = &pair[0];
            let v = &pair[1];
            try!(map.insert(k.clone(), v.clone()));
        }

        Ok(map)
    }

    pub fn insert(&mut self, key: LispValue, value: LispValue) -> EvaluationResult {
        match *key {
            LispType::String(ref s) => {
                self.bindings.insert(s.clone(), value);
                Ok(new_nil())
            }
            LispType::Keyword(ref s) => {
                // will lose keyword discriminant as we only store a string key
                // insert sentinel `:` as a way to read keywords back out of the map
                // need to fix in future
                // let key = format!(":{}", s);
                // self.bindings.insert(key, value);
                self.bindings.insert(s.clone(), value);
                Ok(new_nil())
            }
            _ => Err(EvaluationError::Message("key value is not hashable".to_string())),
        }
    }

    pub fn get(&self, key: &LispValue) -> EvaluationResult {
        match **key {
            LispType::String(ref s) => {
                self.bindings
                    .get(s)
                    .ok_or(EvaluationError::Message("could not find value in map".to_string()))
                    .map(|v| v.clone())
            }
            LispType::Keyword(ref s) => {
                self.bindings
                    .get(s)
                    .ok_or(EvaluationError::Message("could not find value in map".to_string()))
                    .map(|v| v.clone())
            }
            _ => Err(EvaluationError::Message("key value is not hashable".to_string())),
        }
    }

    pub fn contains(&self, key: &LispValue) -> EvaluationResult {
        let exists = match **key {
            LispType::String(ref s) => self.bindings.contains_key(s),
            LispType::Keyword(ref s) => self.bindings.contains_key(s),
            _ => false, // TODO type error on this arm?
        };
        Ok(new_boolean(exists))
    }

    pub fn merge(&mut self, map: &Assoc) -> EvaluationResult {
        for (k, v) in map.bindings.iter() {
            self.bindings.insert(k.clone(), v.clone());
        }
        Ok(new_nil())
    }

    pub fn remove(&mut self, key: &str) {
        let _ = self.bindings.remove(key);
    }

    pub fn keys(&self) -> EvaluationResult {
        let mut result = vec![];
        for key in self.bindings.keys() {
            let next = if key.starts_with(":") {
                new_keyword(&key)
            } else {
                new_string(&key)
            };
            result.push(next);
        }
        Ok(new_list(result))
    }

    pub fn vals(&self) -> EvaluationResult {
        let vals = self.bindings.values().map(|v| v.clone()).collect::<Vec<_>>();
        Ok(new_list(vals))
    }
}

impl fmt::Display for Assoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, (k, v)) in self.bindings.iter().enumerate() {
            try!(write!(f, "{} {}", k, printer::print(v)));
            if i != self.bindings.len() - 1 {
                try!(write!(f, ", "))
            }
        }
        Ok(())
    }
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

pub fn new_vector(s: Seq) -> LispValue {
    value_of(LispType::Vector(s))
}

pub fn new_atom(atom: LispValue) -> LispValue {
    value_of(LispType::Atom(RefCell::new(atom)))
}

pub fn new_keyword(s: &str) -> LispValue {
    value_of(LispType::Keyword(s.to_string()))
}

pub fn new_map(m: Assoc) -> LispValue {
    value_of(LispType::Map(m))
}

pub fn new_map_from_seq(s: Seq) -> EvaluationResult {
    Assoc::from_seq(s).and_then(|assoc| Ok(new_map(assoc)))
}

pub fn new_map_from_fn<F>(m: &Assoc, f: F) -> EvaluationResult
    where F: Fn(String, LispValue) -> Result<(String, LispValue), EvaluationError>
{
    let mut new = Assoc::new();

    for (k, v) in m.bindings.iter() {
        let (k, v) = try!(f(k.clone(), v.clone()));
        new.bindings.insert(k, v);
    }

    Ok(new_map(new))
}

impl fmt::Display for LispType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO need an RC just to satisfy the type here?
        write!(f, "{}", printer::print(&value_of(self.clone())))
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
            (&Vector(ref xs), &Vector(ref ys)) => xs == ys,
            _ => false,
        }
    }
}
