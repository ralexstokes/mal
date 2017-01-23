use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use types::Ast;
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
    pub fn set(&mut self, key: String, val: Ast) {
        self.bindings.insert(key, val);
    }

    pub fn get(&self, key: &str) -> Option<Ast> {
        self.bindings
            .get(key)
            .and_then(|val| Some(val.clone()))
            .or_else(|| {
                if let Some(ref env) = self.outer {
                    env.borrow().get(key)
                } else {
                    None
                }
            })
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
    let one = empty();
    one.borrow_mut().set("a".to_string(), Ast::Symbol("a".to_string()));
    let two = empty_from(one.clone());
    two.borrow_mut().set("b".to_string(), Ast::Symbol("b".to_string()));

    let onea = one.borrow().get(&"a".to_string());
    match onea {
        Some(x) => {
            match x {
                Ast::Symbol(ref s) => assert!(s.as_str() == "a"),
                _ => panic!("wrong ast type"),
            }
        }
        None => panic!("missing binding"),
    }

    let oneb = one.borrow().get(&"b".to_string());
    assert!(oneb.is_none());

    let twob = two.borrow().get(&"b".to_string());
    match twob {
        Some(x) => {
            match x {
                Ast::Symbol(ref s) => assert!(s.as_str() == "b"),
                _ => panic!("wrong ast type"),
            }
        }
        None => panic!("missing binding"),
    }
    let twoa = two.borrow().get(&"a".to_string());
    match twoa {
        Some(x) => {
            match x {
                Ast::Symbol(ref s) => assert!(s.as_str() == "a"),
                _ => panic!("wrong ast type"),
            }
        }
        None => panic!("missing binding"),
    }
}
