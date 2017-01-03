use types::Ast;

pub fn print(ast: Ast) -> Option<String> {
    pr_str(ast, true)
}

// When print_readably is true, doublequotes, newlines, and backslashes are translated into their printed representations (the reverse of the reader).
pub fn pr_str(ast: Ast, readably: bool) -> Option<String> {
    match ast {
        Ast::Nil => Some("nil".to_string()),
        Ast::Boolean(b) => Some(b.to_string()),
        Ast::String(ref s) => {
            if readably {
                Some(unread_str(s))
            } else {
                Some(s.clone())
            }
        }
        Ast::Number(n) => Some(n.to_string()),
        Ast::Symbol(ref s) => Some(s.clone()),
        Ast::List(ref seq) => {
            let results = seq.into_iter()
                .map(|node| pr_str(node.clone(), readably))
                .map(|n| n.unwrap())
                .collect::<Vec<_>>()
                .join(" ");
            ("(".to_string() + &results + ")").into()
        }
        Ast::Lambda { .. } => Some("#<fn>".to_string()),
        Ast::Fn(_) => Some("#<host-fn>".to_string()),
        Ast::Atom(atom) => {
            let inner = atom.borrow();
            Some(format!("atom({})", inner.clone()))
        }
    }
}

// performs the opposite actions of reader::read_str
fn unread_str(s: &str) -> String {
    let mut t = String::new();
    t.push('"');
    for c in s.chars() {
        match c {
            '\n' => t.push_str(r#"\n"#),
            '\\' => t.push_str(r#"\\"#),
            '\"' => t.push_str(r#"\""#),
            _ => t.push(c),
        }
    }
    t.push('"');
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::Ast;

    #[test]
    fn test_print_symbol() {
        let inputstr = "foobar";
        let input = inputstr.to_string();
        let ast = Ast::Symbol(input);
        let output = print(ast).unwrap();
        if output.as_str() != inputstr {
            panic!("not equal")
        }
    }

    #[test]
    fn test_print_number() {
        let ast = Ast::Number(3);
        let output = print(ast).unwrap();
        if output.as_str() != "3" {
            panic!("not equal")
        }
    }

    #[test]
    fn test_print_list() {
        let ast = Ast::List(vec![Ast::Symbol("+".to_string()), Ast::Number(2), Ast::Number(3)]);
        let output = print(ast).unwrap();
        if output.as_str() != "(+ 2 3)" {
            panic!("not equal")
        }
    }
}
