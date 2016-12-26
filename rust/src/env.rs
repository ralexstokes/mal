use std::collections::HashMap;
use std::default::Default;
use types::PrimOpType;

pub struct Env {
    bindings: HashMap<String, PrimOpType>,
}

impl Env {
    pub fn new() -> Env {
        Env::default()
    }

    pub fn lookup(&self, s: &String) -> Option<PrimOpType> {
        match self.bindings.get(s) {
            Some(op) => {
                let op = op.clone();
                Some(op)
            }
            None => None,
        }
    }
}

impl Default for Env {
    fn default() -> Env {
        let mut bindings = HashMap::new();
        bindings.insert("+".to_string(), PrimOpType::Add);
        bindings.insert("-".to_string(), PrimOpType::Subtract);
        bindings.insert("*".to_string(), PrimOpType::Multiply);
        bindings.insert("/".to_string(), PrimOpType::Divide);

        Env { bindings: bindings }
    }
}

pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

pub fn sub(a: i64, b: i64) -> i64 {
    a - b
}

pub fn mul(a: i64, b: i64) -> i64 {
    a * b
}

pub fn div(a: i64, b: i64) -> i64 {
    a / b
}
