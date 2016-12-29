use types::Ast;

pub fn print(ast: Ast) -> Option<String> {
    match ast {
        Ast::Nil => Some("nil".to_string()),
        Ast::Boolean(b) => Some(b.to_string()),
        Ast::String(s) => Some(s),
        Ast::Number(n) => Some(n.to_string()),
        Ast::Symbol(s) => Some(s),
        Ast::List(seq) => {
            let results = seq.into_iter()
                .map(print)
                .map(|node| node.unwrap())
                .collect::<Vec<_>>()
                .join(" ");
            Some("(".to_string() + &results + ")")
        }
        Ast::Lambda { .. } => Some("#<fn>".to_string()),
        Ast::Fn(_) => Some("#<primitive-fn>".to_string()),
    }
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
