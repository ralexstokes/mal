use std::rc::Rc;
use std::cell::RefCell;
use types::Ast;
use ns;

pub type Env = Rc<RefCell<EnvData>>;

#[derive(Debug)]
struct EnvData {
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
    empty_with(None)
}

pub fn empty_with(outer: Option<Env>) -> Env {
    new(outer, ns::Ns::new())
}

impl EnvData {
    pub fn set(&mut self, key: String, val: Ast) {
        self.bindings.insert(key, val);
    }

    pub fn get(&self, key: &String) -> Option<Ast> {
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
}

#[test]
fn test_nesting() {
    let one = empty();
    one.borrow_mut().set("a".to_string(), Ast::Symbol("a".to_string()));
    let two = empty_with(Some(one.clone()));
    two.borrow_mut().set("b".to_string(), Ast::Symbol("b".to_string()));
    assert_eq!(one.borrow().get(&"a".to_string()),
               Some(Ast::Symbol("a".to_string())));
    assert_eq!(one.borrow().get(&"b".to_string()), None);
    assert_eq!(two.borrow().get(&"b".to_string()),
               Some(Ast::Symbol("b".to_string())));
    assert_eq!(two.borrow().get(&"a".to_string()),
               Some(Ast::Symbol("a".to_string())));
}
