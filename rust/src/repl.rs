use reader::read_str;
use printer::pr_str;
use types::Ast;

pub fn rep(input: String) -> Option<String> {
    match read(input) {
        Some(ast) => Some(print(eval(ast))),
        None => None,
    }
}

fn read(input: String) -> Option<Ast> {
    read_str(input)
}

fn eval(ast: Ast) -> Ast {
    ast
}

fn print(ast: Ast) -> String {
    pr_str(ast).unwrap_or("err: could not print output".to_string())
}
