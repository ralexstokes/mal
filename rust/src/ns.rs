use std::collections::HashMap;
use types::{Ast, HostFn};
use printer;

pub type Ns = HashMap<String, Ast>;

pub fn new(bindings: Vec<(String, Ast)>) -> Ns {
    let mut ns = Ns::new();

    for binding in bindings {
        ns.insert(binding.0, binding.1);
    }

    ns
}

pub fn new_from(params: Vec<Ast>, exprs: Vec<Ast>) -> Ns {
    let params = params.iter()
        .map(|p| {
            match *p {
                Ast::Symbol(ref s) => s.clone(),
                _ => unreachable!(),
            }
        })
        .collect::<Vec<_>>();
    let all_params = params.split(|p| p == "&")
        .map(|a| a.clone())
        .collect::<Vec<_>>();

    let mut bound_params = all_params[0].to_vec();

    let mut bound_exprs = exprs.iter()
        .take(bound_params.len())
        .map(|a| a.clone())
        .collect::<Vec<_>>();

    let var_binding = all_params.get(1)
        .and_then(|var_params| var_params.get(0))
        .and_then(|var_param| {
            let var_exprs = exprs.into_iter()
                .skip(bound_exprs.len())
                .collect::<Vec<_>>();
            (var_param.clone(), Ast::List(var_exprs)).into()
        });

    if let Some((param, expr)) = var_binding {
        bound_params.push(param);
        bound_exprs.push(expr);
    }

    let bindings = bound_params.into_iter()
        .zip(bound_exprs.into_iter())
        .map(|(p, e)| (p.clone(), e.clone()))
        .collect::<Vec<_>>();
    new(bindings)
}

pub fn core() -> Ns {
    let mappings: Vec<(&'static str, HostFn)> = vec![("+", add),
                                                     ("-", sub),
                                                     ("*", mul),
                                                     ("/", div),
                                                     ("prn", prn),
                                                     ("pr-str", print_to_str),
                                                     ("str", to_str),
                                                     ("println", println),
                                                     ("list", to_list),
                                                     ("list?", is_list),
                                                     ("empty?", is_empty),
                                                     ("count", count_of),
                                                     ("=", is_equal),
                                                     ("<", lt),
                                                     ("<=", lte),
                                                     (">", gt),
                                                     (">=", gte)];
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
    let pargs = args.into_iter()
        .map(printer::print)
        .map(|p| p.unwrap())
        .collect::<Vec<_>>()
        .join(" ");

    println!("{}", pargs);

    Ast::Nil.into()
}

/*
When a string is read, the following transformations are applied: a backslash followed by a doublequote is translated into a plain doublequote character, a backslash followed by "n" is translated into a newline, and a backslash followed by another backslash is translated into a single backslash.

To properly print a string (for step 4 string functions), the pr_str function needs another parameter called print_readably. When print_readably is true, doublequotes, newlines, and backslashes are translated into their printed representations (the reverse of the reader). The PRINT function in the main program should call pr_str with print_readably set to true.
*/

fn print_to_str(args: Vec<Ast>) -> Option<Ast> {
    let result = args.into_iter()
        .map(|p| printer::print(p))
        .map(|p| p.unwrap())
        .collect::<Vec<_>>()
        .join(" ");
    Ast::String(format!("{:?}",result)).into()
}

// ("str", to_str),
// str: calls pr_str on each argument with print_readably set to false, concatenates the results together ("" separator), and returns the new string.
fn to_str(args: Vec<Ast>) -> Option<Ast> {
    let result = args.iter()
        .map(|p| printer::pr_str(p.clone(), false))
        .map(|p| p.unwrap())
        .collect::<Vec<_>>()
        .join("");

    if result.is_empty() {
        Ast::String(format!("\"{:?}\"", "".to_string())).into()
    } else {
        Ast::String(format!("{}", result)).into()
    }

}

// ("println", println),
// println: calls pr_str on each argument with print_readably set to false, joins the results with " ", prints the string to the screen and then returns nil.
fn println(args: Vec<Ast>) -> Option<Ast> {
    let pargs = args.into_iter()
        .map(|p| printer::pr_str(p, false))
        .map(|p| p.unwrap())
        .collect::<Vec<_>>();

    for p in pargs {
        println!("{}", p);
    }

    Ast::Nil.into()
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
fn is_equal(args: Vec<Ast>) -> Option<Ast> {
    args.split_first()
        .and_then(|(first, rest)| {
            rest.split_first()
                .and_then(|(second, tail)| {
                    if tail.len() != 0 {
                        return None;
                    }

                    is_pair_equal(first.clone(), second.clone()).into()
                })
        })
        .and_then(|result| Ast::Boolean(result).into())
}

fn is_pair_equal(fst: Ast, snd: Ast) -> bool {
    use types::Ast::*;
    match (fst, snd) {
        (Nil, Nil) => true,
        (Boolean(x), Boolean(y)) if x == y => true,
        (String(ref s), String(ref t)) if s == t => true,
        (Number(x), Number(y)) if x == y => true,
        (Symbol(ref s), Symbol(ref t)) if s == t => true,
        (Lambda { .. }, Lambda { .. }) => false,
        (Fn(f), Fn(g)) if f == g => true,
        (List(xs), List(ys)) => {
            xs.len() == ys.len() &&
            xs.iter()
                .zip(ys.iter())
                .map(|(fst, snd)| is_pair_equal(fst.clone(), snd.clone()))
                .all(id)
        }
        _ => false,
    }
}

fn id<T>(x: T) -> T {
    x
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
