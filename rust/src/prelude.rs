use env::Env;
use error::Error;
use repl::Repl;

const IMPLEMENTATION_NAME: &'static str = "mal-rs";

pub fn load(repl: &mut Repl, env: Env) -> Result<(), Error> {
    let host_lang = format!("(def! *host-language* \"{}\")", IMPLEMENTATION_NAME);
    let inputs =
        vec!["(def! not (fn* (a) (if a false true)))",
             "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \")\")))))",
             "(def! *ARGV* (list))",
             "(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> \
              (count xs) 1) (nth xs 1) (throw \"odd number of forms to cond\")) (cons 'cond \
              (rest (rest xs)))))))",
             "(defmacro! or (fn* (& xs) (if (empty? xs) nil (if (= 1 (count xs)) (first xs) \
              `(let* (or_FIXME ~(first xs)) (if or_FIXME or_FIXME (or ~@(rest xs))))))))",
             host_lang.as_str()];
    for input in inputs {
        try!(repl.rep(input.to_string(), env.clone()));
    }
    Ok(())
}
