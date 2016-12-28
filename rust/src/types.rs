use std::fmt;
use env::Env;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug,Clone)]
pub enum Primitive {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug,Clone)]
pub enum Ast {
    Nil,
    Boolean(bool),
    String(String),
    Number(i64),
    Symbol(String),
    If {
        predicate: Box<Ast>,
        consequent: Box<Ast>,
        alternative: Option<Box<Ast>>,
    },
    Do(Vec<Ast>),
    Lambda {
        bindings: Vec<Ast>,
        body: Vec<Ast>,
        env: Rc<RefCell<Env>>,
    },
    Fn(fn(Vec<Ast>) -> Ast),
    Define { name: String, val: Box<Ast> },
    Let { bindings: Vec<Ast>, body: Box<Ast> },
    Combination(Vec<Ast>),
    Operator(Primitive),
}

// Pretty printer for debug
impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Ast::Nil => write!(f, "Nil"),
            Ast::Boolean(ref b) => write!(f, "Boolean({})", b),
            Ast::String(ref s) => write!(f, "String({})", s),
            Ast::Number(ref n) => write!(f, "Number({})", n),
            Ast::Symbol(ref s) => write!(f, "Symbol({})", s),
            Ast::If { predicate: ref p, consequent: ref c, alternative: ref a } => {
                let _ = write!(f, "If({},{})", *p.clone(), *c.clone());
                match *a {
                    Some(ref ast) => {
                        let _ = write!(f, ",{}", *ast.clone());
                    }
                    None => {}
                }
                write!(f, ")")
            }
            Ast::Do(ref seq) => pretty_print_do(f, seq, 0),
            Ast::Lambda { .. } => write!(f, "#<fn>"),
            Ast::Fn(_) => write!(f, "#<primitive-fn>"),
            Ast::Define { name: ref n, val: ref v } => write!(f, "Define({},{})", n, *v),
            Ast::Let { bindings: ref bs, body: ref body } => write!(f, "Let({:?},{:?})", bs, body),
            Ast::Combination(ref seq) => pretty_print_list(f, seq, 0),
            Ast::Operator(_) => unreachable!(),
        }
    }
}

const SPACER: &'static str = "  ";

fn pretty_print_list(f: &mut fmt::Formatter, seq: &Vec<Ast>, depth: i32) -> fmt::Result {
    pretty_print_seq("List", f, seq, depth)
}

fn pretty_print_do(f: &mut fmt::Formatter, seq: &Vec<Ast>, depth: i32) -> fmt::Result {
    pretty_print_seq("Do", f, seq, depth)
}

fn pretty_print_seq(prefix: &'static str,
                    f: &mut fmt::Formatter,
                    seq: &Vec<Ast>,
                    depth: i32)
                    -> fmt::Result {
    let result = write!(f, "\n");
    match result {
        Err(_) => return result,
        _ => {}
    }
    for _ in 0..depth {
        let result = write!(f, "{}", SPACER);
        match result {
            Err(_) => return result,
            _ => {}
        }
    }
    let result = write!(f, "{}([\n", prefix);
    match result {
        Err(_) => return result,
        _ => {}
    }
    for _ in 0..depth + 1 {
        let result = write!(f, "{}", SPACER);
        match result {
            Err(_) => return result,
            _ => {}
        }
    }
    for (i, l) in seq.iter().enumerate() {
        match l {
            &Ast::Combination(ref seq) => {
                let result = pretty_print_list(f, seq, depth + 1);
                match result {
                    Err(_) => return result,
                    _ => {}
                }
            }
            _ => {
                if i != 0 {
                    let result = write!(f, " {},", l);
                    match result {
                        Err(_) => return result,
                        _ => {}
                    }
                } else {
                    let result = write!(f, "{},", l);
                    match result {
                        Err(_) => return result,
                        _ => {}
                    }
                }
            }
        }
    }
    let result = write!(f, "\n");
    match result {
        Err(_) => return result,
        _ => {}
    }
    for _ in 0..depth {
        let result = write!(f, "{}", SPACER);
        match result {
            Err(_) => return result,
            _ => {}
        }
    }
    let result = write!(f, "])");
    match result {
        Err(_) => return result,
        _ => {}
    }
    let result = if depth != 0 { write!(f, ",") } else { Ok(()) };
    result
}
