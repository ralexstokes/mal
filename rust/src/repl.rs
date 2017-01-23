use readline::Reader;
use reader::read;
use printer::print;
use eval::eval;
use env;
use prelude;
use types::Ast;
use error::{Error, ReplError, ReaderError};

pub struct Repl {
    reader: Reader,
}

pub type ReplResult = ::std::result::Result<String, Error>;

const ARGV_SYMBOL: &'static str = "*ARGV*";
const GREETING_FORM: &'static str = "(println (str \"Mal [\" *host-language* \"]\"))";

impl Repl {
    pub fn new(prompt: String) -> Repl {
        Repl { reader: Reader::new(prompt) }
    }

    pub fn run(&mut self) {
        let env = env::core();

        match prelude::load(self, env.clone()) {
            Ok(_) => {}
            Err(Error::ReplError(e)) => {
                match e {
                    ReplError::EmptyOutput => {}
                    ReplError::EvalError(msg) => self.reader.write_err(msg),
                    ReplError::EOF => unreachable!(),
                }
            }
            Err(Error::EvaluationError(e)) => self.reader.write_err(e.to_string()),
            Err(Error::ReaderError(e)) => self.reader.write_err(e.to_string()),
        }

        let args = ::std::env::args().skip(1).collect::<Vec<_>>();
        if !args.is_empty() {
            let result = args.split_first()
                .and_then(|(file_name, env_args)| {
                    let ast_args = env_args.iter()
                        .map(|arg| Ast::Symbol(arg.clone()))
                        .collect::<Vec<_>>();
                    let list_args = Ast::List(ast_args.to_vec());
                    env.borrow_mut().set(ARGV_SYMBOL.to_string(), list_args);
                    self.rep_from_file(file_name, env.clone()).into()
                });
            if let Some(result) = result {
                match result {
                    Ok(result) => println!("{}", result),
                    Err(Error::EvaluationError(e)) => self.reader.write_err(e.to_string()),
                    _ => {}
                }
            }
            return;
        }

        let _ = self.rep(GREETING_FORM.into(), env.clone());

        loop {
            match self.rep_from_reader(env.clone()) {
                Ok(result) => println!("{}", result),
                Err(Error::ReaderError(e)) => {
                    match e {
                        ReaderError::Message(s) => self.reader.write_err(s),
                        ReaderError::EmptyInput => {}
                    }
                }
                Err(Error::EvaluationError(e)) => self.reader.write_err(e.to_string()),
                Err(Error::ReplError(e)) => {
                    match e {
                        ReplError::EmptyOutput => continue,
                        ReplError::EvalError(msg) => self.reader.write_err(msg),
                        ReplError::EOF => break,
                    }
                }
            }
        }
    }

    fn rep_from_reader(&mut self, env: env::Env) -> ReplResult {
        self.reader
            .read()
            .ok_or(Error::ReplError(ReplError::EOF))
            .and_then(|line| self.rep(line, env))
    }

    fn rep_from_file(&mut self, file_name: &str, env: env::Env) -> ReplResult {
        self.rep(format!("(load-file \"{}\")", file_name), env)
    }

    pub fn rep(&mut self, input: String, env: env::Env) -> ReplResult {
        let ast = try!(read(input));
        let val = try!(eval(&ast, env));
        Ok(print(&val))
    }
}
