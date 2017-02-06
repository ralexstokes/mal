use types::{LispValue, LispType, Seq};

pub fn print(value: &LispValue) -> String {
    pr_str(value, true)
}

// When print_readably is true, doublequotes, newlines, and backslashes are translated into their printed representations (the reverse of the reader).
pub fn pr_str(value: &LispValue, readably: bool) -> String {
    match **value {
        LispType::Nil => "nil".to_string(),
        LispType::Boolean(b) => b.to_string(),
        LispType::String(ref s) => {
            if readably {
                unread_str(s)
            } else {
                s.clone()
            }
        }
        LispType::Keyword(ref s) => unread_keyword(s),
        LispType::Number(n) => n.to_string(),
        LispType::Symbol(ref s, ..) => s.clone(),
        LispType::Lambda { .. } => "#<fn>".to_string(),
        LispType::Macro{..} => "#<macro>".to_string(),
        LispType::Fn(..) => "#<host-fn>".to_string(),
        LispType::List(ref seq, ..) => format!("({})", pr_seq(seq, readably)),
        LispType::Vector(ref seq, ..) => format!("[{}]", pr_seq(seq, readably)),
        LispType::Map(ref map, ..) => format!("{{{}}}", map.print(readably)),
        LispType::Atom(ref atom) => {
            let value = atom.borrow();
            format!("(atom {})", *value)
        }
    }
}

fn pr_seq(s: &Seq, readably: bool) -> String {
    s.iter()
        .map(|node| pr_str(node, readably))
        .collect::<Vec<_>>()
        .join(" ")
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

fn unread_keyword(s: &str) -> String {
    // format!(":{}", s);
    // See NOTE about reading keywords
    format!("{}", s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::{new_symbol, new_number, new_list};

    #[test]
    fn test_print_symbol() {
        let input = "foobar";
        let ast = new_symbol(&input, None);
        let output = print(&ast);
        if output.as_str() != input {
            panic!("not equal")
        }
    }

    #[test]
    fn test_print_number() {
        let ast = new_number(3);
        let output = print(&ast);
        if output.as_str() != "3" {
            panic!("not equal")
        }
    }

    #[test]
    fn test_print_list() {
        let ast = new_list(vec![new_symbol("+", None), new_number(2), new_number(3)]);
        let output = print(&ast);
        if output.as_str() != "(+ 2 3)" {
            panic!("not equal")
        }
    }
}
