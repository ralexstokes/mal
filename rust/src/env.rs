use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use types::{LispValue, EvaluationResult};
use error::EvaluationError;
use ns;

pub type Env = Rc<RefCell<EnvData>>;

pub struct EnvData {
    bindings: ns::Ns,
    outer: Option<Env>,
}

pub fn new(outer: Option<Env>, ns: ns::Ns) -> Env {
    Rc::new(RefCell::new(EnvData {
        bindings: ns,
        outer: outer,
    }))
}

pub fn core() -> Env {
    let ns = ns::core();
    new(None, ns)
}

pub fn empty() -> Env {
    new(None, ns::Ns::new())
}

pub fn empty_from(outer: Env) -> Env {
    new(Some(outer), ns::Ns::new())
}

impl EnvData {
    pub fn set(&mut self, key: String, val: LispValue) {
        self.bindings.insert(key, val);
    }

    pub fn get(&self, key: &str) -> EvaluationResult {
        self.find(key)
            .ok_or(EvaluationError::MissingSymbol(key.to_string()))
    }

    fn find(&self, key: &str) -> Option<LispValue> {
        let result = self.bindings.get(key);
        if let Some(value) = result {
            Some(value.clone())
        } else if let Some(ref parent) = self.outer {
            parent.borrow().find(key).clone()
        } else {
            None
        }
    }

    pub fn inspect(&self) {
        for (k, v) in self.bindings.iter() {
            println!("{} => {}", k, v);
        }

        while let Some(ref outer) = self.outer {
            for (k, v) in outer.borrow().bindings.iter() {
                println!("{} => {}", k, v);
            }
        }
    }
}

impl fmt::Debug for EnvData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<env>")
    }
}

impl fmt::Display for EnvData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<env>")
    }
}

pub fn root(env: &Env) -> Env {
    if let Some(ref outer) = env.borrow().outer {
        root(outer)
    } else {
        env.clone()
    }
}

#[test]
fn test_nesting() {
    use types::{LispType, new_symbol};
    let one = empty();
    one.borrow_mut().set("a".to_string(), new_symbol("a", None));
    let two = empty_from(one.clone());
    two.borrow_mut().set("b".to_string(), new_symbol("b", None));

    let onea = one.borrow().get(&"a".to_string());
    match onea {
        Ok(x) => {
            match *x {
                LispType::Symbol(ref s, _) => assert!(s.as_str() == "a"),
                _ => panic!("wrong ast type"),
            }
        }
        Err(_) => panic!("missing binding"),
    }

    let oneb = one.borrow().get(&"b".to_string());
    assert!(oneb.is_err());

    let twob = two.borrow().get(&"b".to_string());
    match twob {
        Ok(x) => {
            match *x {
                LispType::Symbol(ref s, _) => assert!(s.as_str() == "b"),
                _ => panic!("wrong ast type"),
            }
        }
        Err(_) => panic!("could not get binding"),
    }
    let twoa = two.borrow().get(&"a".to_string());
    match twoa {
        Ok(x) => {
            match *x {
                LispType::Symbol(ref s, _) => assert!(s.as_str() == "a"),
                _ => panic!("wrong ast type"),
            }
        }
        Err(_) => panic!("could not get binding"),
    }
}
