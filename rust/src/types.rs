use std::fmt;

#[derive(Debug,Clone)]
pub enum TokenType {
    OpenList,
    CloseList,
    Atom,
    Comment,
}

#[derive(Debug,Clone)]
pub enum Primitive {
    Add,
    Subtract,
    Multiply,
    Divide,

    Define,
    Let,
}

#[derive(Debug,Clone)]
pub enum Ast {
    Symbol(String),
    Number(i64),
    List(Vec<Ast>),
    Operator(Primitive),
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Ast::Symbol(ref s) => write!(f, "Symbol({})", s),
            Ast::Number(ref n) => write!(f, "Number({})", n),
            Ast::List(ref ls) => pretty_print_list(f, ls, 0),
            Ast::Operator(_) => unreachable!(),
        }
    }
}

const SPACER: &'static str = "  ";

fn pretty_print_list(f: &mut fmt::Formatter, ls: &Vec<Ast>, depth: i32) -> fmt::Result {
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
    let result = write!(f, "List([\n");
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
    for (i, l) in ls.iter().enumerate() {
        match l {
            &Ast::List(ref ls) => {
                let result = pretty_print_list(f, ls, depth + 1);
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
