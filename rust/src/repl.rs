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
        let env = Env::core();

        loop {
            let line = self.reader.read();
            match line {
                Some(line) => {
                    let result = read(line);
                    match result {
                        Some(ref ast) => {
                            let result = eval(ast, env.clone())
                                .and_then(print)
                                .unwrap_or("some error".to_string());
                            println!("{}", result);
                        }
                        None => continue,
                    };
                }
                None => break,
            }
        }
    }
}
