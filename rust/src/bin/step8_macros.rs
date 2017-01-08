extern crate mal;

use mal::repl::Repl;
use mal::DEFAULT_PROMPT;

fn main() {
    let prompt = DEFAULT_PROMPT.to_string();
    Repl::new(prompt).run();
}
