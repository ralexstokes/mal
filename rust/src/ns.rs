use std::collections::HashMap;
use std::result::Result;
use types::{Ast, HostFn, EvaluationResult};
use error::{error_message, ReaderError, EvaluationError};
use printer;
use reader;
use std::io::Read;
use std::fs::File;
use eval::{eval, apply_lambda};
use readline;

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
                                                     ("throw", throw),
                                                     ("apply", apply),
                                                     ("map", map),
                                                     ("nil?", is_nil),
                                                     ("true?", is_true),
                                                     ("false?", is_false),
                                                     ("symbol?", is_symbol),
                                                     ("readline", readline),
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

fn fold_first<F>(seq: Vec<Ast>, f: F) -> EvaluationResult
    where F: Fn(Ast, Ast) -> Ast
{
    seq.split_first()
        .ok_or(error_message("not enough args to op"))
        .and_then(|(first, rest)| {
            if rest.len() == 0 {
                return Err(error_message("not enough args to op"))
            }

            let result = rest.iter()
                .map(|a| a.clone())
                .fold(first.clone(), f);

            Ok(result)
        })
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
                                ReaderError::EmptyInput => {
                                    EvaluationError::Message("input was empty".into())
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
                    seq[1..].to_vec()
                };
                Ast::List(items).into()
            },
            Ast::Nil => Some(Ast::Nil),
            _ => None
        }
    }).ok_or(error_message("call to rest failed"))
}

// throw: wraps its argument in an exception
fn throw(args: Vec<Ast>) -> EvaluationResult {
    // println!("evaling throw: {:?}", args);
    let val = if args.len() == 0 {
        Ast::Nil
    } else {
        args[0].clone()
    };

    Err(EvaluationError::Exception((val)))
}


// apply: takes at least two arguments. The first argument is a function and the last argument is list (or vector). The arguments between the function and the last argument (if there are any) are concatenated with the final argument to create the arguments that are used to call the function. The apply function allows a function to be called with arguments that are contained in a list (or vector). In other words, (apply F A B [C D]) is equivalent to (F A B C D).
fn apply(args: Vec<Ast>) -> EvaluationResult {
    let len = args.len();
    if len < 2 {
        return Err(EvaluationError::WrongArity(Ast::Nil)) // fix argument here
    }

    let f = &args[0];
    match f {
        &Ast::Lambda{
            ref env,
            ..
        } => {
            flatten_last(args[1..].to_vec())
                .and_then(|mut args| {
                    let mut app = vec![f.clone()];
                    app.append(&mut args);
                    eval(&Ast::List(app), env.clone())
                })
        },
        &Ast::Fn(f) => {
            flatten_last(args[1..].to_vec())
                .and_then(|args| {
                    f(args)
                })
        }
        _ => Err(error_message("expected first argument to apply to be a function"))
    }
}

// flatten_last returns a vector of elements where each nested element in the last arg in args has been appended to the other arguments in args.
fn flatten_last(args: Vec<Ast>) -> Result<Vec<Ast>, EvaluationError> {
    let mut result = vec![];
    let len = args.len();
    for (i, arg) in args.iter().enumerate() {
        if i == len - 1 {
            match arg {
                &Ast::List(ref seq) => {
                    for s in seq.iter() {
                        result.push(s.clone());
                    }
                },
                _ => return Err(error_message("last argument to apply must be a list"))
            }
            continue
        }

        result.push(arg.clone());
    }
    Ok(result)
}

// map: takes a function and a list (or vector) and evaluates the function against every element of the list (or vector) one at a time and returns the results as a list.
// (map f xs)
fn map(args: Vec<Ast>) -> EvaluationResult {
    args.split_first()
        .ok_or(error_message("wrong arity to map -- missing first arg"))
        .and_then(|(f, rest)| {
            rest.split_first()
                .ok_or(error_message("wrong arity to map -- missing rest args"))
                .and_then(|(xs, _)| {
            match f {
                &Ast::Lambda{
                    ref params,
                    ref body,
                    ref env,
                    ..
                } => {
                    match xs {
                        &Ast::List(ref xs) => {
                            let mut fxs = vec![];
                            for x in xs.iter() {
                                let args = vec![x.clone()];
                                let fx = try!(apply_lambda(params.clone(),
                                                           body.clone(),
                                                           env.clone(),
                                                           args));
                                fxs.push(fx);
                            }
                            Ok(Ast::List(fxs))
                        },
                        _ => Err(error_message("expected second argument to map to be a list or vector"))
                    }
                },
                &Ast::Fn(f) => {
                    match xs {
                        &Ast::List(ref xs) => {
                            let mut fxs = vec![];
                            for x in xs.iter() {
                                let fx = try!(f(vec![x.clone()]));
                                fxs.push(fx);
                            }
                            Ok(Ast::List(fxs))
                        },
                        _ => Err(error_message("expected second argument to map to be a list or vector"))
                    }
                }
                _ => Err(error_message("expected first argument to map to be a lambda value"))
            }
                })
        })
}

// nil?: takes a single argument and returns true (mal true value) if the argument is nil (mal nil value).
fn is_nil(args: Vec<Ast>) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|arg| {
            match *arg {
                Ast::Nil => Ok(Ast::Boolean(true)),
                _ => Ok(Ast::Boolean(false)),
            }
        })
}

// true?: takes a single argument and returns true (mal true value) if the argument is a true value (mal true value).
fn is_true(args: Vec<Ast>) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|arg| {
            match *arg {
                Ast::Boolean(true) => Ok(Ast::Boolean(true)),
                _ => Ok(Ast::Boolean(false)),
            }
        })
}

// false?: takes a single argument and returns true (mal true value) if the argument is a false value (mal false value).
fn is_false(args: Vec<Ast>) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|arg| {
            match *arg {
                Ast::Boolean(false) => Ok(Ast::Boolean(true)),
                _ => Ok(Ast::Boolean(false)),
            }
        })
}

// symbol?: takes a single argument and returns true (mal true value) if the argument is a symbol (mal symbol value).
fn is_symbol(args: Vec<Ast>) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|arg| {
            match *arg {
                Ast::Symbol(_) => Ok(Ast::Boolean(true)),
                _ => Ok(Ast::Boolean(false)),
            }
        })
}

// readline takes a string that is used to prompt the user for input. The line of text entered by the user is returned as a string. If the user sends an end-of-file (usually Ctrl-D), then nil is returned.
fn readline(args: Vec<Ast>) -> EvaluationResult {
    args.first()
        .ok_or(error_message("not enough arguments to readline"))
        .and_then(|arg| {
            match *arg {
                Ast::String(ref s) => {
                    let mut rdr = readline::Reader::new(s.clone());
                    let result = match rdr.read() {
                        Some(line) => Ast::String(line.clone()),
                        None => Ast::Nil,
                    };
                    Ok(result)
                },
                _ => Err(error_message("wrong type of argument to readline"))
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
