use types::{Ast, PrimOpType};

pub fn pr_str(ast: Ast) -> Option<String> {
    match ast {
        Ast::Symbol(s) => Some(s),
        Ast::Number(n) => Some(n.to_string()),
        Ast::List(l) => {
            let results = l.into_iter()
                .map(pr_str)
                .map(|node| node.unwrap())
                .collect::<Vec<_>>()
                .join(" ");
            Some("(".to_string() + &results + ")")
        }
        Ast::PrimOp(op) => {
            let s = match op {
                PrimOpType::Add => "+",
                PrimOpType::Subtract => "-",
                PrimOpType::Multiply => "*",
                PrimOpType::Divide => "/",
            };
            Some(s.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::Ast;

    #[test]
    fn test_pr_str_symbol() {
        let inputstr = "foobar";
        let input = inputstr.to_string();
        let ast = Ast::Symbol(input);
        let output = pr_str(ast).unwrap();
        if output.as_str() != inputstr {
            panic!("not equal")
        }
    }

    #[test]
    fn test_pr_str_number() {
        let ast = Ast::Number(3);
        let output = pr_str(ast).unwrap();
        if output.as_str() != "3" {
            panic!("not equal")
        }
    }

    #[test]
    fn test_pr_str_list() {
        let ast = Ast::List(vec![Ast::Symbol("+".to_string()), Ast::Number(2), Ast::Number(3)]);
        let output = pr_str(ast).unwrap();
        if output.as_str() != "(+ 2 3)" {
            panic!("not equal")
        }
    }
}
