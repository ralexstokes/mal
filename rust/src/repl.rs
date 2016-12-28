use readline::Reader;
use reader::read;
use printer::print;
use eval::eval;
use env::Env;

pub struct Repl {
    reader: Reader,
}

impl Repl {
    pub fn new(prompt: String) -> Repl {
        Repl { reader: Reader::new(prompt) }
    }

    pub fn run(&mut self) {
        self.repl();
    }

    fn repl(&mut self) {
        let mut env = Env::default();

        loop {
            let line = self.reader.read();
            match line {
                Some(line) => {
                    let result = read(line)
                        .and_then(|ref ast| eval(ast, &mut env))
                        .and_then(print)
                        .unwrap_or("some error".to_string());
                    println!("{}", result);
                }
                None => break,
            }
        }
    }
}
