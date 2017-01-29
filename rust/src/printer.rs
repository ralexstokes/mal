use types::{LispValue, LispType};

pub fn print(value: LispValue) -> String {
    pr_str(value, true)
}

// When print_readably is true, doublequotes, newlines, and backslashes are translated into their printed representations (the reverse of the reader).
pub fn pr_str(value: LispValue, readably: bool) -> String {
    match *value {
        LispType::Nil => "nil".to_string(),
        LispType::Boolean(b) => b.to_string(),
        LispType::String(ref s) => {
            if readably {
                unread_str(s)
            } else {
                s.clone()
            }
        }
        LispType::Number(n) => n.to_string(),
        LispType::Symbol(ref s) => s.clone(),
        LispType::List(ref seq) => {
            let results = seq.into_iter()
                .map(|node| pr_str(node.clone(), readably))
                .collect::<Vec<_>>()
                .join(" ");
            ("(".to_string() + &results + ")")
        }
        LispType::Lambda { is_macro, .. } => {
            if is_macro {
                "#<macro>".to_string()
            } else {
                "#<fn>".to_string()
            }
        }
        LispType::Fn(_) => "#<host-fn>".to_string(),
        LispType::Atom(ref atom) => {
            let value = atom.borrow();
            format!("(atom {})", *value)
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
    use types::{new_symbol, new_number, new_list};

    #[test]
    fn test_print_symbol() {
        let input = "foobar";
        let ast = new_symbol(&input);
        let output = print(ast);
        if output.as_str() != input {
            panic!("not equal")
        }
    }

    #[test]
    fn test_print_number() {
        let ast = new_number(3);
        let output = print(ast);
        if output.as_str() != "3" {
            panic!("not equal")
        }
    }

    #[test]
    fn test_print_list() {
        let ast = new_list(vec![new_symbol("+"), new_number(2), new_number(3)]);
        let output = print(ast);
        if output.as_str() != "(+ 2 3)" {
            panic!("not equal")
        }
    }
}
