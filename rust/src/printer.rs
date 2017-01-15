use types::Ast;

pub fn print(ast: &Ast) -> String {
    pr_str(ast, true)
}

// When print_readably is true, doublequotes, newlines, and backslashes are translated into their printed representations (the reverse of the reader).
pub fn pr_str(ast: &Ast, readably: bool) -> String {
    match *ast {
        Ast::Nil => "nil".to_string(),
        Ast::Boolean(b) => b.to_string(),
        Ast::String(ref s) => {
            if readably {
                unread_str(s)
            } else {
                s.clone()
            }
        }
        Ast::Number(n) => n.to_string(),
        Ast::Symbol(ref s) => s.clone(),
        Ast::List(ref seq) => {
            let results = seq.into_iter()
                .map(|node| pr_str(&node, readably))
                .collect::<Vec<_>>()
                .join(" ");
            ("(".to_string() + &results + ")")
        }
        Ast::Lambda { is_macro, .. } => {
            if is_macro {
                "#<macro>".to_string()
            } else {
                "#<fn>".to_string()
            }
        }
        Ast::Fn(_) => "#<host-fn>".to_string(),
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
        let output = print(ast);
        if output.as_str() != inputstr {
            panic!("not equal")
        }
    }

    #[test]
    fn test_print_number() {
        let ast = Ast::Number(3);
        let output = print(ast);
        if output.as_str() != "3" {
            panic!("not equal")
        }
    }

    #[test]
    fn test_print_list() {
        let ast = Ast::List(vec![Ast::Symbol("+".to_string()), Ast::Number(2), Ast::Number(3)]);
        let output = print(ast);
        if output.as_str() != "(+ 2 3)" {
            panic!("not equal")
        }
    }
}
