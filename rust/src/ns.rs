use std::collections::HashMap;
use std::result::Result;
use types::{LispValue, LispType, HostFn, EvaluationResult, Seq, new_list, new_fn, new_number, new_nil, new_string, new_atom, new_boolean, new_symbol, new_lambda, new_macro, new_keyword, new_vector, new_map, new_map_from_seq, Assoc};
use error::{error_message, ReaderError, EvaluationError};
use printer;
use reader;
use std::io::Read;
use std::fs::File;
use eval::{eval, apply_lambda};
use readline;
use time;

pub type Ns = HashMap<String, LispValue>;

pub fn new(bindings: Vec<(String, LispValue)>) -> Ns {
    let mut ns = Ns::new();

    for binding in bindings {
        ns.insert(binding.0, binding.1);
    }

    ns
}

pub fn new_from(params: Seq, exprs: Seq) -> Ns {
    let params = params.iter()
        .map(|p| {
            match **p {
                LispType::Symbol(ref s, ..) => s.clone(),
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
            (var_param.clone(), new_list(var_exprs, None)).into()
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
                                                     ("atom", to_atom),
                                                     ("atom?", is_atom),
                                                     ("deref", deref),
                                                     ("reset!", reset),
                                                     ("swap!", swap),
                                                     ("symbol", to_symbol),
                                                     ("keyword", to_keyword),
                                                     ("keyword?", is_keyword),
                                                     ("vector", to_vector),
                                                     ("vector?", is_vector),
                                                     ("sequential?", is_seq),
                                                     ("hash-map", to_map),
                                                     ("map?", is_map),
                                                     ("assoc", assoc),
                                                     ("dissoc", dissoc),
                                                     ("get", get),
                                                     ("contains?", contains),
                                                     ("keys", keys),
                                                     ("vals", vals),
                                                     ("time-ms", time_millis),
                                                     ("conj", conj),
                                                     ("string?", is_string),
                                                     ("seq", to_seq),
                                                     ("meta", meta_of),
                                                     ("with-meta", with_meta)
    ];
    let bindings = mappings.iter()
        .map(|&(k, v)| (k.to_string(), new_fn(v, None)))
        .collect();
    new(bindings)
}

fn i64_from_ast(a: LispValue, b: LispValue) -> (i64, i64) {
    let aa = match *a {
        LispType::Number(x) => x,
        _ => 0,
    };

    let bb = match *b {
        LispType::Number(x) => x,
        _ => 0,
    };

    (aa, bb)
}

fn fold_first<F>(seq: Seq, f: F) -> EvaluationResult
    where F: Fn(LispValue, LispValue) -> LispValue
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

fn add(seq: Seq) -> EvaluationResult {
    fold_first(seq, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        new_number(a + b)
    })
}

fn sub(seq: Seq) -> EvaluationResult {
    fold_first(seq, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        new_number(a - b)
    })
}

fn mul(seq: Seq) -> EvaluationResult {
    fold_first(seq, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        new_number(a * b)
    })
}

fn div(seq: Seq) -> EvaluationResult {
    fold_first(seq, |a, b| {
        let (a, b) = i64_from_ast(a, b);
        new_number(a / b)
    })
}

fn string_of(args: Seq, readably: bool, separator: &str) -> String {
    args.iter()
        .map(|p| printer::pr_str(p, readably))
        .collect::<Vec<_>>()
        .join(separator)
}

// prn: calls pr_str on each argument with print_readably set to true, joins the results with " ", prints the string to the screen and then returns nil.
fn prn(args: Seq) -> EvaluationResult {
    println!("{}", string_of(args, true, " "));

    Ok(new_nil())
}

// pr-str: calls pr_str on each argument with print_readably set to true, joins the results with " " and returns the new string.
fn print_to_str(args: Seq) -> EvaluationResult {
    let s = string_of(args, true, " ");

    Ok(new_string(&s))
}

// str: calls pr_str on each argument with print_readably set to false, concatenates the results together ("" separator), and returns the new string.
fn to_str(args: Seq) -> EvaluationResult {
    let s = string_of(args, false, "");

    Ok(new_string(&s))
}

// println: calls pr_str on each argument with print_readably set to false, joins the results with " ", prints the string to the screen and then returns nil.
fn println(args: Seq) -> EvaluationResult {
    println!("{}", string_of(args, false, " "));

    Ok(new_nil())
}

fn to_list(args: Seq) -> EvaluationResult {
    Ok(new_list(args, None))
}

fn is_list(args: Seq) -> EvaluationResult {
    args.first()
        .and_then(|a| {
            let is = match **a {
                LispType::List(..) => true,
                _ => false,
            };
            new_boolean(is).into()
        })
        .ok_or(error_message("could not determine if seq is list"))
}

fn is_empty(args: Seq) -> EvaluationResult {
    args.first()
        .and_then(|a| {
            match **a {
                LispType::List(ref seq, ..) |
                LispType::Vector(ref seq, ..)=> new_boolean(seq.is_empty()).into(),
                _ => None,
            }
        })
        .ok_or(error_message("could not determine if seq is empty"))
}

fn count_of(args: Seq) -> EvaluationResult {
    args.first()
        .and_then(|a| {
            match **a {
                LispType::List(ref seq, ..) |
                LispType::Vector(ref seq, ..) => new_number(seq.len() as i64).into(),
                LispType::Nil => new_number(0).into(),
                _ => None,
            }
        })
        .ok_or(error_message("could not determine count of seq"))
}

fn is_equal(args: Seq) -> EvaluationResult {
    args.split_first()
        .and_then(|(first, rest)| {
            rest.split_first()
                .and_then(|(second, tail)| {
                    if tail.len() != 0 {
                        return None;
                    }

                    new_boolean(first == second).into()
                })
        })
        .ok_or(error_message("could not determine if args are equal"))
}

fn args_are<F>(args: Seq, f: F) -> EvaluationResult
    where F: Fn(i64, i64) -> bool
{
    args.split_first()
        .and_then(|(first, rest)| {
            rest.split_first()
                .and_then(|(second, _)| {
                    let (a, b) = i64_from_ast(first.clone(), second.clone());
                    new_boolean(f(a, b)).into()
                })
        }).ok_or(error_message("could not determine if args are ordered"))
}


fn lt(args: Seq) -> EvaluationResult {
    args_are(args, |a, b| a < b)
}


fn lte(args: Seq) -> EvaluationResult {
    args_are(args, |a, b| a <= b)
}


fn gt(args: Seq) -> EvaluationResult {
    args_are(args, |a, b| a > b)
}


fn gte(args: Seq) -> EvaluationResult {
    args_are(args, |a, b| a >= b)
}


fn read_string(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("not enough arg in args"))
        .and_then(|arg| {
            match **arg {
                LispType::String(ref s) => {
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

fn slurp(args: Seq) -> EvaluationResult {
    let filename = args.first()
        .and_then(|arg| {
            match **arg {
                LispType::String(ref filename) => filename.into(),
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
                    new_string(&buffer)
                }).map_err(|_| error_message("slurp could not read file"))
        })
}

// cons: this function takes a list as its second parameter and returns a new list that has the first argument prepended to it.
fn cons(args: Seq) -> EvaluationResult {
    args.split_first()
        .and_then(|(elem, rest)| {
            rest.split_first()
                .and_then(|(list, _)| {
                    let mut elems = vec![elem.clone()];
                    match **list {
                        LispType::List(ref seq, ..) |
                        LispType::Vector(ref seq, ..)=> {
                            for s in seq {
                                elems.push(s.clone())
                            }

                            new_list(elems, None).into()
                        },
                        _ => None
                    }
                })
        })
        .ok_or(error_message("call to cons failed"))
}

// concat: this functions takes 0 or more lists as parameters and returns a new list that is a concatenation of all the list parameters.
fn concat(args: Seq) -> EvaluationResult {
    let mut result: Seq = vec![];

    for arg in args {
        match *arg {
            LispType::List(ref seq, ..) |
            LispType::Vector(ref seq, ..) => {
                for s in seq {
                    result.push(s.clone());
                }
            },
            _ => return Err(error_message("wrong arg type for concat"))
        }
    }

    Ok(new_list(result, None))
}

// nth: this function takes a list (or vector) and a number (index) as arguments, returns the element of the list at the given index. If the index is out of range, this function raises an exception.
fn nth(args: Seq) -> EvaluationResult {
    let result = args.split_first().and_then(|(seq, rest)| {
        rest.split_first().and_then(|(idx, _)| {
            match **seq {
                LispType::List(ref seq, ..) |
                LispType::Vector(ref seq, ..) => {
                    match **idx {
                        LispType::Number(n) => {
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
fn first(args: Seq) -> EvaluationResult {
    args.first().and_then(|seq| {
        match **seq {
            LispType::List(ref seq, ..) |
            LispType::Vector(ref seq, ..) => {
                if seq.is_empty() {
                    Some(new_nil())
                } else {
                    seq.first().map(|elem| elem.clone())
                }
            },
            LispType::Nil => Some(new_nil()),
            _ => None
        }
    }).ok_or(error_message("call to first failed"))
}

// rest: this function takes a list (or vector) as its argument and returns a new list containing all the elements except the first.
fn rest(args: Seq) -> EvaluationResult {
    args.first().and_then(|seq| {
        match **seq {
            LispType::List(ref seq, ..) |
            LispType::Vector(ref seq, ..) => {
                let items = if seq.is_empty() {
                    vec![]
                } else {
                    seq[1..].to_vec()
                };
                new_list(items, None).into()
            },
            LispType::Nil => Some(new_list(vec![], None)),
            _ => None
        }
    }).ok_or(error_message("call to rest failed"))
}

// throw: wraps its argument in an exception
fn throw(args: Seq) -> EvaluationResult {
    let val = if args.len() == 0 {
        new_nil()
    } else {
        args[0].clone()
    };

    Err(EvaluationError::Exception((val)))
}


// apply: takes at least two arguments. The first argument is a function and the last argument is list (or vector). The arguments between the function and the last argument (if there are any) are concatenated with the final argument to create the arguments that are used to call the function. The apply function allows a function to be called with arguments that are contained in a list (or vector). In other words, (apply F A B [C D]) is equivalent to (F A B C D).
fn apply(args: Seq) -> EvaluationResult {
    let len = args.len();
    if len < 2 {
        return Err(EvaluationError::WrongArity(new_nil())) // fix argument here
    }

    let f = &args[0];
    match **f {
        LispType::Lambda{
            ref env,
            ..
        } => {
            flatten_last(args[1..].to_vec())
                .and_then(|mut args| {
                    let mut app = vec![f.clone()];
                    app.append(&mut args);
                    eval(new_list(app, None), env.clone())
                })
        },
        LispType::Fn(f, ..) => {
            flatten_last(args[1..].to_vec())
                .and_then(|args| {
                    f(args)
                })
        }
        _ => Err(error_message("expected first argument to apply to be a function"))
    }
}

// flatten_last returns a vector of elements where each nested element in the last arg in args has been appended to the other arguments in args.
fn flatten_last(args: Seq) -> Result<Seq, EvaluationError> {
    let mut result = vec![];
    let len = args.len();
    for (i, arg) in args.iter().enumerate() {
        if i == len - 1 {
            match **arg {
                LispType::List(ref seq, ..) |
                LispType::Vector(ref seq, ..) => {
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
fn map(args: Seq) -> EvaluationResult {
    args.split_first()
        .ok_or(error_message("wrong arity to map -- missing first arg"))
        .and_then(|(f, rest)| {
            rest.split_first()
                .ok_or(error_message("wrong arity to map -- missing rest args"))
                .and_then(|(xs, _)| {
            match **f {
                LispType::Lambda{
                    ref params,
                    ref body,
                    ref env,
                    ..
                } => {
                    match **xs {
                        LispType::List(ref xs, ..) |
                        LispType::Vector(ref xs, ..) => {
                            let mut fxs = vec![];
                            for x in xs.iter() {
                                let args = vec![x.clone()];
                                let fx = try!(apply_lambda(params.clone(),
                                                           body.clone(),
                                                           env.clone(),
                                                           args));
                                fxs.push(fx);
                            }
                            Ok(new_list(fxs, None))
                        },
                        _ => Err(error_message("expected second argument to map to be a list or vector"))
                    }
                },
                LispType::Fn(f, ..) => {
                    match **xs {
                        LispType::List(ref xs, ..) |
                        LispType::Vector(ref xs, ..) => {
                            let mut fxs = vec![];
                            for x in xs.iter() {
                                let fx = try!(f(vec![x.clone()]));
                                fxs.push(fx);
                            }
                            Ok(new_list(fxs, None))
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
fn is_nil(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|arg| {
            match **arg {
                LispType::Nil => Ok(new_boolean(true)),
                _ => Ok(new_boolean(false)),
            }
        })
}

// true?: takes a single argument and returns true (mal true value) if the argument is a true value (mal true value).
fn is_true(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|arg| {
            match **arg {
                LispType::Boolean(true) => Ok(new_boolean(true)),
                _ => Ok(new_boolean(false)),
            }
        })
}

// false?: takes a single argument and returns true (mal true value) if the argument is a false value (mal false value).
fn is_false(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|arg| {
            match **arg {
                LispType::Boolean(false) => Ok(new_boolean(true)),
                _ => Ok(new_boolean(false)),
            }
        })
}

// symbol?: takes a single argument and returns true (mal true value) if the argument is a symbol (mal symbol value).
fn is_symbol(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|arg| {
            match **arg {
                LispType::Symbol(..) => Ok(new_boolean(true)),
                _ => Ok(new_boolean(false)),
            }
        })
}

// readline takes a string that is used to prompt the user for input. The line of text entered by the user is returned as a string. If the user sends an end-of-file (usually Ctrl-D), then nil is returned.
fn readline(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("not enough arguments to readline"))
        .and_then(|arg| {
            match **arg {
                LispType::String(ref s) => {
                    let mut rdr = readline::Reader::new(s);
                    let result = match rdr.read() {
                        Some(line) => new_string(&line),
                        None => new_nil(),
                    };
                    Ok(result)
                },
                _ => Err(error_message("wrong type of argument to readline"))
            }
        })
}


// atom: Takes a Mal value and returns a new atom which points to that Mal value.
fn to_atom(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|arg| {
            Ok(new_atom(arg.clone()))
        })
}

// atom?: Takes an argument and returns true if the argument is an atom.
fn is_atom(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|a| {
            let is_atom = if let LispType::Atom(_) = **a {
                true
            } else {
                false
            };
            Ok(new_boolean(is_atom))
        })
}

// deref: Takes an atom argument and returns the Mal value referenced by this atom.
fn deref(args: Seq) -> EvaluationResult {
    if args.len() == 0 {
        return Err(EvaluationError::WrongArity(new_nil())) // fix argument here
    }

    let arg = &args[0];
    match **arg {
        LispType::Atom(ref atom) => {
            let val = atom.borrow();
            Ok(val.clone())
        }
        _ => Err(error_message("wrong type of argument to deref"))
    }
}

// reset!: Takes an atom and a Mal value; the atom is modified to refer to the given Mal value. The Mal value is returned.
fn reset(args: Seq) -> EvaluationResult {
    args.split_first()
        .ok_or(error_message("wrong arity"))
        .and_then(|(atom, rest)| {
            rest.split_first()
                .ok_or(error_message("wrong arity"))
                .and_then(|(val, _)| {
                    match **atom {
                        LispType::Atom(ref atomr) => {
                            let mut atom = atomr.borrow_mut();
                            *atom = val.clone();
                            Ok(val.clone())
                        }
                        _ => Err(error_message("wrong type of argument to reset!"))
                    }
        })
    })
}

// swap!: Takes an atom, a function, and zero or more function arguments. The atom's value is modified to the result of applying the function with the atom's value as the first argument and the optionally given function arguments as the rest of the arguments. The new atom's value is returned.
// (swap! myatom (fn* [x y] (+ 1 x y)) 22)
fn swap(args: Seq) -> EvaluationResult {
    args.split_first()
        .ok_or(error_message("wrong arity to swap!"))
        .and_then(|(atom, rest)| {
            rest.split_first()
                .ok_or(error_message("wrong arity to swap!"))
                .and_then(|(f, args)| {
                    match **atom {
                        LispType::Atom(ref atom) => {
                            match **f {
                                LispType::Lambda{
                                    ref params,
                                    ref body,
                                    ref env,
                                    ..
                                } => {
                                    let mut value = atom.borrow_mut();
                                    let mut full_params = vec![value.clone()];
                                    full_params.append(&mut args.to_vec());
                                    apply_lambda(params.clone(), body.clone(), env.clone(), full_params.to_vec()).and_then(|newval| {
                                        *value = newval.clone();
                                        Ok(newval)
                                    })
                                },
                                LispType::Fn(f, ..) => {
                                    let mut value = atom.borrow_mut();
                                    let mut full_params = vec![value.clone()];
                                    full_params.append(&mut args.to_vec());
                                    f(full_params).and_then(|newval| {
                                        *value = newval.clone();
                                        Ok(newval)
                                    })
                                }
                                _ => Err(error_message("wrong type of arguments to swap!")),
                            }
                        },
                        _ => Err(error_message("wrong type of arguments to swap!"))
                    }
                })
        })
}

// ** symbol: takes a string and returns a new symbol with the string as its name.
fn to_symbol(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|s| {
            if let LispType::String(ref s) = **s {
                Ok(new_symbol(s, None))
            } else {
                Err(error_message("wrong type to symbol"))
            }
        })
}

// ** keyword: takes a string and returns a keyword with the same name (usually just be prepending the special keyword unicode symbol). This function should also detect if the argument is already a keyword and just return it.
fn to_keyword(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|k| {
            if let LispType::String(ref s) = **k {
                // NOTE: see note about keyword handling
                Ok(new_keyword(&format!(":{}", s)))
            } else if let LispType::Keyword(ref s) = **k {
                Ok(new_keyword(s))
            } else {
                Err(error_message("wrong type to keyword"))
            }
        })
}

// ** keyword?: takes a single argument and returns true (mal true value) if the argument is a keyword, otherwise returns false (mal false value).
fn is_keyword(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|k| {
            let is_keyword = if let LispType::Keyword(_) = **k {
                true
            } else {
                false
            };
            Ok(new_boolean(is_keyword))
        })
}

// vector: takes a variable number of arguments and returns a vector containing those arguments.
fn to_vector(args: Seq) -> EvaluationResult {
    Ok(new_vector(args, None))
}

// vector?: takes a single argument and returns true (mal true value) if the argument is a vector, otherwise returns false (mal false value).
fn is_vector(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|v| {
            let is_vector = if let LispType::Vector(..) = **v {
                true
            } else {
                false
            };
            Ok(new_boolean(is_vector))
        })
}

// sequential?: takes a single arguments and returns true (mal true value) if it is a list or a vector, otherwise returns false (mal false value).
fn is_seq(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|s| {
            let is_seq = match **s {
                LispType::List(..) => true,
                LispType::Vector(..) => true,
                _ => false,
            };
            Ok(new_boolean(is_seq))
        })
}

// hash-map: takes a variable but even number of arguments and returns a new mal hash-map value with keys from the odd arguments and values from the even arguments respectively. This is basically the functional form of the {} reader literal syntax.
fn to_map(args: Seq) -> EvaluationResult {
    new_map_from_seq(args)
}

// map?: takes a single argument and returns true (mal true value) if the argument is a hash-map, otherwise returns false (mal false value).
fn is_map(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity"))
        .and_then(|m| {
            let is_assoc = match **m {
                LispType::Map(..) => true,
                _ => false,
            };
            Ok(new_boolean(is_assoc))
        })
}

// assoc: takes a hash-map as the first argument and the remaining arguments are odd/even key/value pairs to "associate" (merge) into the hash-map. Note that the original hash-map is unchanged (remember, mal values are immutable), and a new hash-map containing the old hash-maps key/values plus the merged key/value arguments is returned.
fn assoc(args: Seq) -> EvaluationResult {
    args.split_first()
        .ok_or(error_message("wrong arity to assoc"))
        .and_then(|(map, rest)| {
            match **map {
                LispType::Map(ref map, ..) => {
                    let mut new = map.clone();
                    let rest = try!(Assoc::from_seq(rest.to_vec()));
                    try!(new.merge(&rest));
                    Ok(new_map(new, None))
                },
                _ => Err(error_message("wrong type of arguments to assoc"))
            }
        })
}

// dissoc: takes a hash-map and a list of keys to remove from the hash-map. Again, note that the original hash-map is unchanged and a new hash-map with the keys removed is returned. Key arguments that do not exist in the hash-map are ignored.
fn dissoc(args: Seq) -> EvaluationResult {
    args.split_first()
        .ok_or(error_message("wrong arity to dissoc"))
        .and_then(|(map, keys)| {
            match **map {
                LispType::Map(ref map, ..) => {
                    let mut new = map.clone();

                    for key in keys.iter() {
                        match **key {
                            LispType::String(ref s) |
                            LispType::Keyword(ref s) => {
                                new.remove(s);
                            }
                            _ => return Err(error_message("wrong type of arguments to dissoc"))
                        }
                    }

                    Ok(new_map(new, None))
                },
                _ => Err(error_message("wrong type of arguments to dissoc"))
            }
        })
}

// get: takes a hash-map and a key and returns the value of looking up that key in the hash-map. If the key is not found in the hash-map then nil is returned.
fn get(args: Seq) -> EvaluationResult {
    args.split_first()
        .ok_or(error_message("wrong arity to get"))
        .and_then(|(map, rest)| {
            rest.split_first()
                .ok_or(error_message("wrong arity to get"))
                .and_then(|(key, _)| {
                    match **map {
                        LispType::Map(ref map, ..) => {
                            map.get(key).or(Ok(new_nil()))
                        },
                        LispType::Nil => {
                            Ok(new_nil())
                        }
                        _ => Err(error_message("wrong type of arguments to get"))
                    }
                })
        })
}

// contains?: takes a hash-map and a key and returns true (mal true value) if the key exists in the hash-map and false (mal false value) otherwise.
fn contains(args: Seq) -> EvaluationResult {
    args.split_first()
        .ok_or(error_message("wrong arity to contains?"))
        .and_then(|(map, rest)| {
            rest.split_first()
                .ok_or(error_message("wrong arity to contains?"))
                .and_then(|(key, _)| {
                    match **map {
                        LispType::Map(ref map, ..) => {
                            map.contains(key)
                        },
                        _ => Err(error_message("wrong type of arguments to contains?"))
                    }
                })
        })
}

// keys: takes a hash-map and returns a list (mal list value) of all the keys in the hash-map.
fn keys(args: Seq) -> EvaluationResult {
    args.split_first()
        .ok_or(error_message("wrong arity to keys"))
        .and_then(|(map, _)| {
            match **map {
                LispType::Map(ref map, ..) => {
                    map.keys()
                },
                _ => Err(error_message("wrong type of arguments to keys"))
            }
        })
}
// vals: takes a hash-map and returns a list (mal list value) of all the values in the hash-map.
fn vals(args: Seq) -> EvaluationResult {
    args.split_first()
        .ok_or(error_message("wrong arity to vals"))
        .and_then(|(map, _)| {
            match **map {
                LispType::Map(ref map, ..) => {
                    map.vals()
                },
                _ => Err(error_message("wrong type of arguments to vals"))
            }
        })
}

// time-ms: takes no arguments and returns the number of milliseconds since epoch (00:00:00 UTC January 1, 1970).
fn time_millis(_: Seq) -> EvaluationResult {
    let t = time::get_time();
    let sec = t.sec as i64;
    let nsec = t.nsec as i64;
    let millis = sec * 1000;
    let nsec_millis = nsec / 1_000_000;
    let total_millis = millis + nsec_millis;
    Ok(new_number(total_millis))
}

// conj: takes a collection and one or more elements as arguments and returns a new collection which includes the original collection and the new elements. If the collection is a list, a new list is returned with the elements inserted at the start of the given list in opposite order; if the collection is a vector, a new vector is returned with the elements added to the end of the given vector.
fn conj(args: Seq) -> EvaluationResult {
    args.split_first()
        .ok_or(error_message("wrong arity to conj"))
        .and_then(|(coll, rest)| {
            match **coll {
                LispType::List(ref seq, ..) => {
                    let mut new_seq = vec![];

                    for elem in rest.iter().rev() {
                        new_seq.push(elem.clone());
                    }

                    for elem in seq.iter() {
                        new_seq.push(elem.clone());
                    }

                    Ok(new_list(new_seq, None))
                },
                LispType::Vector(ref seq, ..) => {
                    let mut new_seq = vec![];
                    for elem in seq.iter() {
                        new_seq.push(elem.clone());
                    }
                    for elem in rest.iter() {
                        new_seq.push(elem.clone());
                    }
                    Ok(new_vector(new_seq, None))
                }
                _ => Err(error_message("wrong type of arguments to conj"))
            }
        })
}

// string?: returns true if the parameter is a string.
fn is_string(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity to string?"))
        .and_then(|m| {
            let is_string = match **m {
                LispType::String(_) => true,
                _ => false,
            };
            Ok(new_boolean(is_string))
        })
}

// seq: takes a list, vector, string, or nil. If an empty list, empty vector, or empty string ("") is passed in then nil is returned. Otherwise, a list is returned unchanged, a vector is converted into a list, and a string is converted to a list that containing the original string split into single character strings.
fn to_seq(args: Seq) -> EvaluationResult {
    args.first()
        .and_then(|a| {
            match **a {
                LispType::List(ref seq, ..) |
                LispType::Vector(ref seq, ..) => {
                    if seq.is_empty() {
                        Some(new_nil())
                    } else {
                        Some(new_list(seq.clone(), None))
                    }
                }
                LispType::String(ref s) => {
                    if s.is_empty() {
                        Some(new_nil())
                    } else {
                        let mut seq = vec![];
                        for c in s.chars() {
                            let next = new_string(&c.to_string());
                            seq.push(next);
                        }
                        Some(new_list(seq, None))
                    }
                }
                LispType::Nil => new_nil().into(),
                _ => None,
            }
        })
        .ok_or(error_message("could not seq this value"))
}

// meta :: Value -> Metadata
// gets metadata from value
fn meta_of(args: Seq) -> EvaluationResult {
    args.first()
        .ok_or(error_message("wrong arity to meta"))
        .and_then(|value| {
            match **value {
                LispType::Symbol(_, ref metadata) => {
                    Ok(metadata.clone())
                },
                LispType::Lambda{ref metadata, ..} => {
                    Ok(metadata.clone())
                },
                LispType::Macro{ref metadata, ..} => {
                    Ok(metadata.clone())
                },
                LispType::Fn(_, ref metadata) => {
                    Ok(metadata.clone())
                }
                LispType::List(_, ref metadata) => {
                    Ok(metadata.clone())
                }
                LispType::Vector(_, ref metadata) => {
                    Ok(metadata.clone())
                }
                LispType::Map(_, ref metadata) => {
                    Ok(metadata.clone())
                }
                _ => Ok(new_nil()),
            }
        })
}

// with-meta :: Value -> Map -> Value
// creates value with supplied metadata
fn with_meta(args: Seq) -> EvaluationResult {
    args.split_first()
        .ok_or(error_message("wrong arity to with-meta"))
        .and_then(|(value, rest)| {
            rest.split_first()
                .ok_or(error_message("wrong arity to with-meta"))
                .and_then(|(meta, _)| {
                    // uncomment if we only want maps as metadata
                    // if let LispType::Map(ref meta, ..) = **meta {
                        // let meta = new_metadata(meta.clone());
                    match **value {
                        LispType::Symbol(ref s, ..) => {
                            Ok(new_symbol(s, Some(meta.clone())))
                        },
                        LispType::Lambda{
                            ref params,
                            ref body,
                            ref env, ..} => {
                            Ok(new_lambda(params.clone(), body.clone(), env.clone(), Some(meta.clone())))
                        },
                        LispType::Macro{
                            ref params,
                            ref body,
                            ref env, ..} => {
                            Ok(new_macro(params.clone(), body.clone(), env.clone(), Some(meta.clone())))
                        },
                        LispType::Fn(f, ..) => {
                            Ok(new_fn(f, Some(meta.clone())))
                        }
                        LispType::List(ref seq, ..) => {
                            Ok(new_list(seq.clone(), Some(meta.clone())))
                        }
                        LispType::Vector(ref seq, ..) => {
                            Ok(new_vector(seq.clone(), Some(meta.clone())))
                        }
                        LispType::Map(ref assoc, ..) => {
                            Ok(new_map(assoc.clone(), Some(meta.clone())))
                        }
                        _ => Err(error_message("type of this argument does not support metadata")),
                    }
                    // } else {
                    //     Err(error_message("metadata must be of type map"))
                    // }
                })
        })
}
