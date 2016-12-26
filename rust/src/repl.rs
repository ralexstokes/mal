use reader::read_str;
use printer::pr_str;
use types::{Ast, PrimOpType};
use env::{Env, add, sub, mul, div};

pub fn rep(input: String, env: &Env) -> Option<String> {
    read(input)
        .and_then(|ast| eval(ast, env))
        .map(print)
}

fn read(input: String) -> Option<Ast> {
    read_str(input)
}

fn eval(ast: Ast, env: &Env) -> Option<Ast> {
    match ast {
        Ast::Number(n) => Some(Ast::Number(n)),
        Ast::Symbol(s) => env.lookup(s).map(Ast::PrimOp),
        Ast::List(ls) => {
            let els = ls.into_iter()
                .map(|l| eval(l, env))
                .filter(|l| l.is_some())
                .map(|l| l.unwrap())
                .collect::<Vec<_>>();
            if let Some((op, ops)) = els.split_first() {
                match *op {
                    Ast::PrimOp(ref op) => {
                        let operands = ops.to_vec();
                        apply(op, operands)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Ast::PrimOp(_) => unreachable!(),
    }
}

fn apply(op: &PrimOpType, args: Vec<Ast>) -> Option<Ast> {
    let f = match op {
        &PrimOpType::Add => add,
        &PrimOpType::Subtract => sub,
        &PrimOpType::Multiply => mul,
        &PrimOpType::Divide => div,
    };
    let nums = args.iter()
        .map(|arg| match arg {
            &Ast::Number(n) => n,
            _ => 0,
        })
        .collect::<Vec<_>>();
    if let Some((first, rest)) = nums.split_first() {
        let mut result = *first;
        for &n in rest {
            result = f(result, n);
        }
        Some(Ast::Number(result))
    } else {
        None
    }
}

fn print(ast: Ast) -> String {
    pr_str(ast).unwrap_or("".to_string())
}
