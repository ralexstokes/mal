use readline::Reader;
use repl::rep;
use env::{Env, add_default_bindings};

pub struct Driver {
    reader: Reader,
}

impl Driver {
    pub fn new(prompt: String) -> Driver {
        Driver { reader: Reader::new(prompt) }
    }

    pub fn run(&mut self) {
        self.repl();
    }

    fn repl(&mut self) {
        let mut env = Env::new(None);
        add_default_bindings(&mut env);

        loop {
            let input = self.reader.read();
            match input {
                Some(line) => {
                    match rep(line, &mut env) {
                        Some(output) => println!("{}", output),
                        None => println!("some error"),
                    }
                }
                None => break,
            }
        }
    }
}
