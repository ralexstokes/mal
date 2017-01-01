use readline::Reader;
use reader::read;
use printer::print;
use eval::eval;
use env;
use prelude;
use types::Ast;

pub struct Repl {
    reader: Reader,
}

const ARGV_SYMBOL: &'static str = "*ARGV*";

impl Repl {
    pub fn new(prompt: String) -> Repl {
        Repl { reader: Reader::new(prompt) }
    }

    pub fn run(&mut self) {
        let env = env::core();

        let mut pretext: Option<String> = None;
        match prelude::load(self, env.clone()) {
            Ok(msg) => pretext = msg.clone().into(),
            Err(Error::EmptyOutput) => {}
            Err(Error::EvalError(msg)) => self.reader.write_err(msg),
            Err(Error::EOF) => unreachable!(),
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
                    Err(Error::EvalError(msg)) => println!("{}", msg),
                    _ => {}
                }
            }
            return;
        }

        if let Some(msg) = pretext {
            self.reader.write_ok(msg)
        }

        loop {
            match self.rep_from_reader(env.clone()) {
                Ok(result) => println!("{}", result),
                Err(Error::EmptyOutput) => continue,
                Err(Error::EvalError(msg)) => println!("{}", msg),
                Err(Error::EOF) => break,
            }
        }
    }

    fn rep_from_reader(&mut self, env: env::Env) -> Result<String, Error> {
        match self.reader.read() {
            Some(line) => self.rep(line, env),
            None => Err(Error::EOF),
        }
    }

    fn rep_from_file(&mut self, file_name: &str, env: env::Env) -> Result<String, Error> {
        self.rep(format!("(load-file \"{}\")", file_name), env)
    }

    pub fn rep(&mut self, input: String, env: env::Env) -> Result<String, Error> {
        match read(input) {
            Some(ref ast) => {
                eval(ast, env.clone())
                    .and_then(print)
                    .ok_or_else(|| Error::EvalError("some error".to_string()))
            }
            None => Err(Error::EmptyOutput),
        }
    }
}

pub enum Error {
    EmptyOutput,
    EvalError(String),
    EOF,
}
