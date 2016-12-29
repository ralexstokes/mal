use std::collections::HashMap;
use types::{Ast, HostFn};
use printer::print;

pub type Ns = HashMap<String, Ast>;

pub fn new(bindings: Vec<(String, Ast)>) -> Ns {
    let mut ns = Ns::new();

    for binding in bindings {
        ns.insert(binding.0, binding.1);
    }

    ns
}

pub fn core() -> Ns {
    let mappings: Vec<(&'static str, HostFn)> = vec![("+", add),
                                                     // ("-", sub),
                                                     // ("*", mul),
                                                     // ("/", div),
                                                     // ("prn", prn),
                                                     // ("list", to_list),
                                                     // ("list?", is_list),
                                                     // ("empty?", is_empty),
                                                     // ("count", count_of),
                                                     // ("=", is_eq),
                                                     // ("<", lt),
                                                     // ("<=", lte),
                                                     // (">", gt),
                                                     // (">=", gte)
    ];
    let bindings = mappings.iter()
        .map(|&(k, v)| (k.to_string(), Ast::Fn(v)))
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

fn fold_first<F>(xs: Vec<Ast>, f: F) -> Option<Ast>
    where F: Fn(Ast, Ast) -> Ast
{
    xs.split_first()
        .and_then(|(first, rest)| {
            let result = rest.iter()
                .map(|a| a.clone())
                .fold(first.clone(), f);
            Some(result)
        })
}

fn add(xs: Vec<Ast>) -> Option<Ast> {
    fold_first(xs, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a + b)
    })
}

fn sub(xs: Vec<Ast>) -> Option<Ast> {
    fold_first(xs, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a - b)
    })
}

fn mul(xs: Vec<Ast>) -> Option<Ast> {
    fold_first(xs, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a * b)
    })
}

fn div(xs: Vec<Ast>) -> Option<Ast> {
    fold_first(xs, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a / b)
    })
}

fn prn(args: Vec<Ast>) -> Option<Ast> {
    args.first()
        .and_then(|a| print(a.clone()))
        .and_then(|s| {
            println!("{}", s);
            Ast::Nil.into()
        })
}

fn to_list(args: Vec<Ast>) -> Option<Ast> {
    Ast::List(args).into()
}

fn is_list(args: Vec<Ast>) -> Option<Ast> {
    args.first()
        .and_then(|a| {
            let is = match a.clone() {
                Ast::List(_) => true,
                _ => false,
            };
            Ast::Boolean(is).into()
        })
}

fn is_empty(args: Vec<Ast>) -> Option<Ast> {
    args.first()
        .and_then(|a| {
            match a.clone() {
                Ast::List(seq) => Ast::Boolean(seq.is_empty()).into(),
                _ => None,
            }
        })
}

fn count_of(args: Vec<Ast>) -> Option<Ast> {
    args.first()
        .and_then(|a| {
            match a.clone() {
                Ast::List(seq) => Ast::Number(seq.len() as i64).into(),
                Ast::Nil => Ast::Number(0).into(),
                _ => None,
            }
        })
}
fn is_eq(args: Vec<Ast>) -> Option<Ast> {
    None
    // use types::Ast::*;

    // let pair = args.split_first()
    //     .and_then(|(first, rest)| {
    //         rest.split_first()
    //             .and_then(|(second, _)| Some((first, second)))
    //     });

    // pair.and_then(|(first, second)| {
    //     match (first, second) {

    //     }
    // })

    // if let Some((first, second)) = pair {
    //     Ast::Boolean(first == second).into()
    // }
}

fn args_are<F>(args: Vec<Ast>, f: F) -> Option<Ast>
    where F: Fn(i64, i64) -> bool
{
    args.split_first()
        .and_then(|(first, rest)| {
            rest.split_first()
                .and_then(|(second, _)| {
                    let (a, b) = i64_from_ast(first.clone(), second.clone());
                    Ast::Boolean(f(a, b)).into()
                })
        })
}

fn lt(args: Vec<Ast>) -> Option<Ast> {
    args_are(args, |a, b| a < b)
}

fn lte(args: Vec<Ast>) -> Option<Ast> {
    args_are(args, |a, b| a <= b)
}

fn gt(args: Vec<Ast>) -> Option<Ast> {
    args_are(args, |a, b| a > b)
}

fn gte(args: Vec<Ast>) -> Option<Ast> {
    args_are(args, |a, b| a >= b)
}
