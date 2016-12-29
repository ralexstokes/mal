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
        self.bindings.insert(Ast::Symbol(key), val);
    }

    // pub fn find(self, key: &String) -> Option<Box<Env>> {
    //     if self.bindings.contains_key(key) {
    //         Some(Box::new(self))
    //     } else {
    //         if let Some(env) = self.outer {
    //             env.find(key)
    //         } else {
    //             None
    //         }
    //     }
    // }

    pub fn get(&self, key: &String) -> Option<Ast> {
        self.bindings
            .get(&Ast::Symbol(key.clone()))
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
