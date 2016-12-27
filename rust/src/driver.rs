use readline::Reader;
use repl::rep;
use env::Env;

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
        let env = Env::new();
        loop {
            let input = self.reader.read();
            match input {
                Some(line) => {
                    match rep(line, &env) {
                        Some(output) => println!("{}", output),
                        None => println!("some error"),
                    }
                }
                None => break,
            }
        }
    }
}
