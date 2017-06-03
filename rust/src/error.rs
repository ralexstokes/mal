use std::fmt;
use std::convert::From;
use reader::Token;
use types::LispValue;

#[derive(Debug)]
pub enum EvaluationError {
    WrongArity(LispValue),
    BadArguments(LispValue),
    MissingSymbol(String),
    Message(String),
    Exception(LispValue),
}

impl From<EvaluationError> for Error {
    fn from(err: EvaluationError) -> Error {
        Error::EvaluationError(err)
    }
}

pub fn error_message(msg: &str) -> EvaluationError {
    EvaluationError::Message(msg.to_string())
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EvaluationError::WrongArity(ref ast) => {
                write!(f, "TODO -- wrong number of arguments for fn: {}", ast)
            }
            EvaluationError::BadArguments(ref ast) => {
                write!(f, "TODO -- bad inputs for fn: {}", ast)
            }
            EvaluationError::Message(ref s) => write!(f, "{}", s),
            EvaluationError::MissingSymbol(ref s) => write!(f, "unbound symbol: {}", s),
            EvaluationError::Exception(_) => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum ReaderError {
    Message(String),
    ExtraInput(Vec<Token>, usize),
    EmptyInput,
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReaderError::Message(ref s) => write!(f, "{}", s),
            ReaderError::ExtraInput(_, _) => {
                write!(f, "extra input after full form")
                // let _ = write!(f, "extra input after full form: (whitespace removed, sorry)\n");
                // for token in tokens {
                //     let _ = match token {
                //         &Token::List(Direction::Open) => write!(f, "("),
                //         &Token::List(Direction::Close) => write!(f, ")"),
                //         &Token::Vector(Direction::Open) => write!(f, "["),
                //         &Token::Vector(Direction::Close) => write!(f, "]"),
                //         &Token::Map(Direction::Open) => write!(f, "{{"),
                //         &Token::Map(Direction::Close) => write!(f, "}}"),
                //         &Token::Atom(ref s) => write!(f, "{}", s),
                //         &Token::Comment(ref s) => write!(f, "{}", s),
                //     };
                // }
                // let _ = write!(f, "\n",);
                // let mut pos = pos as isize;
                // while pos >= 0 {
                //     pos -= 1;
                //     let _ = write!(f, " ");
                // }
                // write!(f, "^")
            }
            ReaderError::EmptyInput => Ok(()),
        }
    }
}

impl From<ReaderError> for Error {
    fn from(err: ReaderError) -> Error {
        Error::ReaderError(err)
    }
}


pub enum ReplError {
    EmptyOutput,
    EvalError(String),
    EOF,
}

impl fmt::Display for ReplError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReplError::EmptyOutput => Ok(()),
            ReplError::EvalError(ref s) => write!(f, "{}", s),
            ReplError::EOF => Ok(()),
        }
    }
}

impl From<ReplError> for Error {
    fn from(err: ReplError) -> Error {
        Error::ReplError(err)
    }
}

pub enum Error {
    ReaderError(ReaderError),
    EvaluationError(EvaluationError),
    ReplError(ReplError),
}
