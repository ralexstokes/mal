extern crate rustyline;
extern crate regex;
#[macro_use]
extern crate lazy_static;

pub const DEFAULT_PROMPT: &'static str = "user> ";

pub mod driver;
pub mod readline;
pub mod repl;
pub mod reader;
pub mod types;
pub mod printer;
