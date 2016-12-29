use std::collections::HashMap;
use types::Ast;

pub type Ns = HashMap<Ast, Ast>;

pub fn new(bindings: Vec<(Ast, Ast)>) -> Ns {
    let mut ns = Ns::new();

    for binding in bindings {
        ns.insert(binding.0, binding.1);
    }

    ns
}

type AstReducer = fn(Vec<Ast>) -> Ast;
pub fn core() -> Ns {
    let mappings: Vec<(&'static str, AstReducer)> = vec![("+", add), ("-", sub), ("*", mul),
                                                         ("/", div)];
    let bindings = mappings.iter()
        .map(|&(k, v)| (Ast::Symbol(k.to_string()), Ast::Fn(v)))
        .collect();
    new(bindings)
}

fn i64_from_ast(a: Ast, b: Ast) -> (i64, i64) {
    let aa = match a {
        Ast::Number(x) => x,
        _ => 0,
    };

    let bb = match b {
        Ast::Number(x) => x,
        _ => 0,
    };

    (aa, bb)
}

fn fold_first<F>(xs: Vec<Ast>, f: F) -> Ast
    where F: Fn(Ast, Ast) -> Ast
{
    xs.split_first()
        .and_then(|(first, rest)| {
            let result = rest.iter()
                .map(|a| a.clone())
                .fold(first.clone(), f);
            Some(result)
        })
        .unwrap_or(Ast::Nil)
}

fn add(xs: Vec<Ast>) -> Ast {
    fold_first(xs, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a + b)
    })
}

fn sub(xs: Vec<Ast>) -> Ast {
    fold_first(xs, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a - b)
    })
}

fn mul(xs: Vec<Ast>) -> Ast {
    fold_first(xs, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a * b)
    })
}

fn div(xs: Vec<Ast>) -> Ast {
    fold_first(xs, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a / b)
    })
}
