use readline::Reader;
use repl::rep;

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
        loop {
            let input = self.reader.read();
            match input {
                Some(line) => {
                    match rep(line) {
                        Some(output) => println!("{}", output),
                        None => print!(""),
                    }
                }
                None => break,
            }
        }
    }
}
