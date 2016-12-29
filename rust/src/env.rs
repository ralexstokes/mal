use std::rc::Rc;
use std::cell::RefCell;
use types::Ast;
use ns;

#[derive(Debug)]
pub struct Env {
    bindings: ns::Ns,
    outer: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new(outer: Option<Rc<RefCell<Env>>>, ns: ns::Ns) -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env {
            bindings: ns,
            outer: outer,
        }))
    }

    pub fn core() -> Rc<RefCell<Env>> {
        let ns = ns::core();
        Self::new(None, ns)
    }

    pub fn empty() -> Rc<RefCell<Env>> {
        Self::empty_with(None)
    }

    pub fn empty_with(outer: Option<Rc<RefCell<Env>>>) -> Rc<RefCell<Env>> {
        Self::new(outer, ns::Ns::new())
    }

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
    let one = Env::empty();
    one.borrow_mut().set("a".to_string(), Ast::Symbol("a".to_string()));
    let two = Env::empty_with(Some(one.clone()));
    two.borrow_mut().set("b".to_string(), Ast::Symbol("b".to_string()));
    assert_eq!(one.borrow().get(&"a".to_string()),
               Some(Ast::Symbol("a".to_string())));
    assert_eq!(one.borrow().get(&"b".to_string()), None);
    assert_eq!(two.borrow().get(&"b".to_string()),
               Some(Ast::Symbol("b".to_string())));
    assert_eq!(two.borrow().get(&"a".to_string()),
               Some(Ast::Symbol("a".to_string())));
}
