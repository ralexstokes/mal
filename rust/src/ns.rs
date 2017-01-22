use std::collections::HashMap;
use types::{Ast, HostFn, EvaluationResult};
use error::{error_message, ReaderError, EvaluationError};
use printer;
use reader;
use std::io::Read;
use std::fs::File;
use eval::eval;

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
                                                     (">=", gte),
                                                     ("read-string", read_string),
                                                     ("slurp", slurp),
                                                     ("cons", cons),
                                                     ("concat", concat),
                                                     ("nth", nth),
                                                     ("first", first),
                                                     ("rest", rest),
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

fn fold_first<F>(seq: Vec<Ast>, f: F) -> EvaluationResult
    where F: Fn(Ast, Ast) -> Ast
{
    seq.split_first()
        .and_then(|(first, rest)| {
            let result = rest.iter()
                .map(|a| a.clone())
                .fold(first.clone(), f);
            Some(result)
        })
        .ok_or(error_message("could not calculate op on seq"))
}

fn add(seq: Vec<Ast>) -> EvaluationResult {
    fold_first(seq, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a + b)
    })
}

fn sub(seq: Vec<Ast>) -> EvaluationResult {
    fold_first(seq, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a - b)
    })
}

fn mul(seq: Vec<Ast>) -> EvaluationResult {
    fold_first(seq, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a * b)
    })
}

fn div(seq: Vec<Ast>) -> EvaluationResult {
    fold_first(seq, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        Ast::Number(a / b)
    })
}

fn string_of(args: Vec<Ast>, readably: bool, separator: &str) -> String {
    args.iter()
        .map(|p| printer::pr_str(p, readably))
        .collect::<Vec<_>>()
        .join(separator)
}

// prn: calls pr_str on each argument with print_readably set to true, joins the results with " ", prints the string to the screen and then returns nil.
fn prn(args: Vec<Ast>) -> EvaluationResult {
    println!("{}", string_of(args, true, " "));

    Ok(Ast::Nil)
}

// pr-str: calls pr_str on each argument with print_readably set to true, joins the results with " " and returns the new string.
fn print_to_str(args: Vec<Ast>) -> EvaluationResult {
    let s = string_of(args, true, " ");

    Ok(Ast::String(s))
}

// str: calls pr_str on each argument with print_readably set to false, concatenates the results together ("" separator), and returns the new string.
fn to_str(args: Vec<Ast>) -> EvaluationResult {
    let s = string_of(args, false, "");

    Ok(Ast::String(s))
}

// println: calls pr_str on each argument with print_readably set to false, joins the results with " ", prints the string to the screen and then returns nil.
fn println(args: Vec<Ast>) -> EvaluationResult {
    println!("{}", string_of(args, false, " "));

    Ok(Ast::Nil)
}

fn to_list(args: Vec<Ast>) -> EvaluationResult {
    Ok(Ast::List(args))
}

fn is_list(args: Vec<Ast>) -> EvaluationResult {
    args.first()
        .and_then(|a| {
            let is = match a.clone() {
                Ast::List(_) => true,
                _ => false,
            };
            Ast::Boolean(is).into()
        })
        .ok_or(error_message("could not determine if seq is list"))
}

fn is_empty(args: Vec<Ast>) -> EvaluationResult {
    args.first()
        .and_then(|a| {
            match a.clone() {
                Ast::List(seq) => Ast::Boolean(seq.is_empty()).into(),
                _ => None,
            }
        })
        .ok_or(error_message("could not determine if seq is empty"))
}

fn count_of(args: Vec<Ast>) -> EvaluationResult {
    args.first()
        .and_then(|a| {
            match a.clone() {
                Ast::List(seq) => Ast::Number(seq.len() as i64).into(),
                Ast::Nil => Ast::Number(0).into(),
                _ => None,
            }
        })
        .ok_or(error_message("could not determine count of seq"))
}

fn is_equal(args: Vec<Ast>) -> EvaluationResult {
    args.split_first()
        .and_then(|(first, rest)| {
            rest.split_first()
                .and_then(|(second, tail)| {
                    if tail.len() != 0 {
                        return None;
                    }

                    Ast::Boolean(first == second).into()
                })
        })
        .ok_or(error_message("could not determine if args are equal"))
}

fn args_are<F>(args: Vec<Ast>, f: F) -> EvaluationResult
    where F: Fn(i64, i64) -> bool
{
    args.split_first()
        .and_then(|(first, rest)| {
            rest.split_first()
                .and_then(|(second, _)| {
                    let (a, b) = i64_from_ast(first.clone(), second.clone());
                    Ast::Boolean(f(a, b)).into()
                })
        }).ok_or(error_message("could not determine if args are ordered"))
}


fn lt(args: Vec<Ast>) -> EvaluationResult {
    args_are(args, |a, b| a < b)
}


fn lte(args: Vec<Ast>) -> EvaluationResult {
    args_are(args, |a, b| a <= b)
}


fn gt(args: Vec<Ast>) -> EvaluationResult {
    args_are(args, |a, b| a > b)
}


fn gte(args: Vec<Ast>) -> EvaluationResult {
    args_are(args, |a, b| a >= b)
}


fn read_string(args: Vec<Ast>) -> EvaluationResult {
    args.first()
        .ok_or(error_message("not enough arg in args"))
        .and_then(|arg| {
            match *arg {
                Ast::String(ref s) => {
                    reader::read(s.clone()).map_err(|e| {
                            match e {
                                ReaderError::Message(s) => {
                                    EvaluationError::Message(s)
                                }
                            }
                        })
                },
                _ => Err(error_message("wrong type of first argument"))
            }
        })
}

fn slurp(args: Vec<Ast>) -> EvaluationResult {
    let filename = args.first()
        .and_then(|arg| {
            match *arg {
                Ast::String(ref filename) => filename.into(),
                _ => None
            }
        });

    let mut buffer = String::new();
    filename
        .ok_or(error_message("slurp could not get filename from args"))
        .and_then(|filename| {
            File::open(filename)
                .and_then(|mut f| {
                    f.read_to_string(&mut buffer)
                }).map(|_| {
                    Ast::String(buffer)
                }).map_err(|_| error_message("slurp could not read file"))
        })
}

// cons: this function takes a list as its second parameter and returns a new list that has the first argument prepended to it.
fn cons(args: Vec<Ast>) -> EvaluationResult {
    args.split_first()
        .and_then(|(elem, rest)| {
            rest.split_first()
                .and_then(|(list, _)| {
                    let mut elems = vec![elem.clone()];
                    match *list {
                        Ast::List(ref seq) => {
                            for s in seq {
                                elems.push(s.clone())
                            }

                            Ast::List(elems).into()
                        },
                        _ => None
                    }
                })
        })
        .ok_or(error_message("call to cons failed"))
}

// concat: this functions takes 0 or more lists as parameters and returns a new list that is a concatenation of all the list parameters.
fn concat(args: Vec<Ast>) -> EvaluationResult {
    let mut result: Vec<Ast> = vec![];

    for arg in args {
        match arg {
            Ast::List(ref seq) => {
                for s in seq {
                    result.push(s.clone());
                }
            },
            _ => return Err(error_message("wrong arg type for concat"))
        }
    }

    Ok(Ast::List(result))
}

// nth: this function takes a list (or vector) and a number (index) as arguments, returns the element of the list at the given index. If the index is out of range, this function raises an exception.
fn nth(args: Vec<Ast>) -> EvaluationResult {
    let result = args.split_first().and_then(|(seq, rest)| {
        rest.split_first().and_then(|(idx, _)| {
            match *seq {
                Ast::List(ref seq) => {
                    match *idx {
                        Ast::Number(n) => {
                            let n = n as usize;
                            seq.get(n)
                        },
                        _ => None
                    }
                },
                _ => None
            }
        })
    });
    result.and_then(|result| result.clone().into())
        .ok_or(error_message("call to nth failed"))
}

// first: this function takes a list (or vector) as its argument and return the first element. If the list (or vector) is empty or is nil then nil is returned.
fn first(args: Vec<Ast>) -> EvaluationResult {
    args.first().and_then(|seq| {
        match *seq {
            Ast::List(ref seq)  => {
                if seq.is_empty() {
                    Some(Ast::Nil)
                } else {
                    seq.first().map(|elem| elem.clone())
                }
            },
            Ast::Nil => Some(Ast::Nil),
            _ => None
        }
    }).ok_or(error_message("call to first failed"))
}

// rest: this function takes a list (or vector) as its argument and returns a new list containing all the elements except the first.
fn rest(args: Vec<Ast>) -> EvaluationResult {
    args.first().and_then(|seq| {
        match *seq {
            Ast::List(ref seq)  => {
                let items = if seq.is_empty() {
                    vec![]
                } else {
                    seq[1..].to_vec()//.iter().map(|elem| elem.clone()).collect::<Vec<_>>()
                };
                Ast::List(items).into()
            },
            Ast::Nil => Some(Ast::Nil),
            _ => None
        }
    }).ok_or(error_message("call to rest failed"))
}
