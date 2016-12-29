extern crate rustyline;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate term;

pub const DEFAULT_PROMPT: &'static str = "user> ";

pub mod repl;
pub mod readline;
pub mod eval;
pub mod reader;
pub mod types;
pub mod printer;
pub mod env;
pub mod ns;
pub mod prelude;
