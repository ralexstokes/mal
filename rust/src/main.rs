// Add the 4 trivial functions READ, EVAL, PRINT, and rep (read-eval-print). READ, EVAL, and PRINT are basically just stubs that return their first parameter (a string if your target language is a statically typed) and rep calls them in order passing the return to the input of the next.

//     Add a main loop that repeatedly prints a prompt (needs to be "user> " for later tests to pass), gets a line of input from the user, calls rep with that line of input, and then prints out the result from rep. It should also exit when you send it an EOF (often Ctrl-D).


extern crate rusty;
use rusty::{driver};

const DEFAULT_PROMPT: &'static str = "user> ";

fn main() {
    let prompt = DEFAULT_PROMPT.to_string();

    driver::run(prompt);
}
