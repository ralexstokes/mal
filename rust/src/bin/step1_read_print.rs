extern crate mal;

use mal::driver::Driver;
use mal::DEFAULT_PROMPT;

fn main() {
    let prompt = DEFAULT_PROMPT.to_string();
    let mut driver = Driver::new(prompt);
    driver.run();
}
