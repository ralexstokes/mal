use std::collections::HashMap;
use std::default::Default;
use types::PrimOpType;

// type FnMap = HashMap<String, Fn(i64, i64) -> i64>;

pub struct Env {
    bindings: HashMap<String, PrimOpType>, // Box<Fn(i64, i64) -> i64 + 'a>>
}

impl Env {
    pub fn new() -> Env {
        Env::default()
    }

    pub fn lookup(&self, s: String) -> Option<PrimOpType> {
        match self.bindings.get(&s) {
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
