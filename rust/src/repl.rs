use readline::Reader;
use reader::read;
use printer::print;
use eval::eval;
use env;
use prelude;

pub struct Repl {
    reader: Reader,
}

impl Repl {
    pub fn new(prompt: String) -> Repl {
        Repl { reader: Reader::new(prompt) }
    }

    pub fn run(&mut self) {
        let env = env::core();

        match prelude::load(self, env.clone()) {
            Ok(msg) => self.reader.write_ok(msg),
            Err(Error::EmptyOutput) => {}
            Err(Error::EvalError(msg)) => self.reader.write_err(msg),
            Err(Error::EOF) => unreachable!(),
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
        let line = self.reader.read();
        match line {
            Some(line) => self.rep(line, env),
            None => Err(Error::EOF),
        }
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
