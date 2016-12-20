use readline;
use repl;

pub fn run(prompt: String) {
    loop {
        repl(prompt);
        break;
    }
}

fn repl(prompt: String) {
    let input = readline::prompt(prompt);
    let output = repl::rep(input);
    println!("{}", output);
}
