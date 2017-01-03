use std::collections::HashMap;
use types::{Ast, HostFn};
use printer;
use reader;
use std::io::Read;
use std::fs::File;
use std::cell::RefCell;
use eval::{eval_ops, apply};

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
                                                     ("atom", atom),
                                                     ("atom?", is_atom),
                                                     ("deref", deref),
                                                     ("reset!", reset),
                                                     ("swap!", swap),
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

fn string_of(args: Vec<Ast>, readably: bool, separator: &str) -> String {
    args.into_iter()
        .map(|p| printer::pr_str(p, readably))
        .map(|p| p.unwrap())
        .collect::<Vec<_>>()
        .join(separator)
}

// prn: calls pr_str on each argument with print_readably set to true, joins the results with " ", prints the string to the screen and then returns nil.
fn prn(args: Vec<Ast>) -> Option<Ast> {
    println!("{}", string_of(args, true, " "));

    Ast::Nil.into()
}

// pr-str: calls pr_str on each argument with print_readably set to true, joins the results with " " and returns the new string.
fn print_to_str(args: Vec<Ast>) -> Option<Ast> {
    let s = string_of(args, true, " ");

    Ast::String(s).into()
}

// str: calls pr_str on each argument with print_readably set to false, concatenates the results together ("" separator), and returns the new string.
fn to_str(args: Vec<Ast>) -> Option<Ast> {
    let s = string_of(args, false, "");

    Ast::String(s).into()
}

// println: calls pr_str on each argument with print_readably set to false, joins the results with " ", prints the string to the screen and then returns nil.
fn println(args: Vec<Ast>) -> Option<Ast> {
    println!("{}", string_of(args, false, " "));

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


fn read_string(args: Vec<Ast>) -> Option<Ast> {
    args.first()
        .and_then(|arg| {
            match *arg {
                Ast::String(ref s) => {
                    reader::read(s.clone())
                },
                _ => None
            }
        })
}

fn slurp(args: Vec<Ast>) -> Option<Ast> {
    args.first()
        .and_then(|arg| {
            match *arg {
                Ast::String(ref filename) => {
                    let mut buffer = String::new();
                    let result = File::open(filename)
                        .and_then(|mut f| {
                            f.read_to_string(&mut buffer)
                        });
                    match result {
                        Ok(_) => Some(Ast::String(buffer)),
                        Err(e) => {
                            println!("{}", e);
                            None
                        }
                    }
                },
                _ => None
            }
        })
}


// atom: Takes a Mal value and returns a new atom which points to that Mal value.
fn atom(args: Vec<Ast>) -> Option<Ast> {
    args.first().and_then(|arg| {
        Ast::Atom(RefCell::new(Box::new(arg.clone()))).into()
    })
}

// atom?: Takes an argument and returns true if the argument is an atom.
fn is_atom(args: Vec<Ast>) -> Option<Ast> {
    args.first()
        .and_then(|a| {
            let is = match a.clone() {
                Ast::Atom(_) => true,
                _ => false,
            };
            Ast::Boolean(is).into()
        })
}

// deref: Takes an atom argument and returns the Mal value referenced by this atom.
fn deref(args: Vec<Ast>) -> Option<Ast> {
    if args.len() == 0 {
        return None;
    }

    let arg = args[0].clone();
    match arg {
        Ast::Atom(atom) => {
            let val = atom.into_inner();
            Some(*val)
        }
        _ => None,
    }
}

// reset!: Takes an atom and a Mal value; the atom is modified to refer to the given Mal value. The Mal value is returned.
fn reset(args: Vec<Ast>) -> Option<Ast> {
    args.split_first().and_then(|(atom, rest)| {
        rest.split_first().and_then(|(val, _)| {
            println!("{:?}", val);
            let newval = Box::new(val.clone());
            println!("{:?}", newval);

            match atom {
                &Ast::Atom(ref atom) => {
                    println!("{:?}", atom);
                    let mut cell = atom.borrow_mut();
                    let oldval = (*cell).clone();
                    *cell = newval;
                    println!("{:?}", cell);
                    Some(*oldval)
                }
                _ => None,
            }
        })
    })
}

// swap!: Takes an atom, a function, and zero or more function arguments. The atom's value is modified to the result of applying the function with the atom's value as the first argument and the optionally given function arguments as the rest of the arguments. The new atom's value is returned. (Side note: Mal is single-threaded, but in concurrent languages like Clojure, swap! promises atomic update: (swap! myatom (fn* [x] (+ 1 x))) will always increase the myatom counter by one and will not suffer from missing updates when the atom is updated from multiple threads.)
// (swap! myatom (fn* [x y] (+ 1 x y)) 22)
fn swap(args: Vec<Ast>) -> Option<Ast> {
    args.split_first().and_then(|(atom, rest)| {
        rest.split_first().and_then(|(f, params)| {
            match atom {
                &Ast::Atom(ref atom) => {
                    match f {
                        &Ast::Lambda{
                            ref env,
                            ..
                        } => {
                            let val = atom.clone().into_inner();
                            let mut full_params = vec![*val];
                            full_params.append(&mut params.to_vec());
                            let evops = eval_ops(full_params, env.clone());
                            apply(f, evops, env.clone()).and_then(|newval| {
                                let newvalcopy = newval.clone();
                                let mut a = atom.borrow_mut();
                                *a = Box::new(newvalcopy);
                                newval.into()
                            })
                        },
                        _ => None,
                    }
                },
                _ => None
            }
        })
    })
}
