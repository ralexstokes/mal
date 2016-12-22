use reader::read_str;
use printer::pr_str;
use types::Ast;

pub fn rep(input: String) -> String {
    print(eval(read(input)))
}

fn read(input: String) -> Ast {
    read_str(input)
}

fn eval(ast: Ast) -> Ast {
    ast
}

fn print(ast: Ast) -> String {
    pr_str(ast).unwrap_or("err: could not print output".to_string())
}
