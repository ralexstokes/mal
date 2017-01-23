use std::fmt;
use std::convert::From;
use types::Ast;

#[derive(Debug)]
pub enum EvaluationError {
    WrongArity(Ast),
    BadArguments(Ast),
    MissingSymbol(String),
    Message(String),
    Exception(Ast),
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
    EmptyInput,
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReaderError::Message(ref s) => write!(f, "{}", s),
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
