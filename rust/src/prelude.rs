use env::Env;
use repl::{Repl, Error};

pub fn load(repl: &mut Repl, env: Env) -> Result<String, Error> {
    let inputs = vec!["(def! not (fn* (a) (if a false true)))",
                      "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \
                       \")\")))))",
                      "(def! *ARGV* (list))"];
    for input in inputs {
        try!(repl.rep(input.to_string(), env.clone()));
    }
    Ok("~ prelude loaded ~".to_string())
}
