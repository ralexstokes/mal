use std::io::{self, Write};

pub fn prompt(prompt: String) -> String {
    print!("{}", prompt);

    io::stdout().flush().expect("could not flush to stdout");

    let mut input = String::new();

    io::stdin().read_line(&mut input)
        .expect("failed to read input");

    input.trim().to_string()
}
