use regex::{Regex, Captures};
use types::{LispValue, new_symbol, new_list, new_nil, new_boolean, new_number, new_string,
            new_keyword};
use error::ReaderError;

pub type ReaderResult = ::std::result::Result<LispValue, ReaderError>;

pub fn read(input: String) -> ReaderResult {
    let tokens = tokenizer(input);
    let mut reader = Reader::new(tokens);
    read_form(&mut reader)
}

const TOKEN_REGEX: &'static str =
    r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"|;.*|[^\s\[\]{}('"`,;)]*)"#;

fn tokenizer(input: String) -> Vec<Token> {
    lazy_static! {
        static ref RE: Regex = Regex::new(TOKEN_REGEX).unwrap();
    }
    RE.captures_iter(&input)
        .map(token_from)
        .collect::<Vec<_>>()
}

#[test]
fn test_tokenizer() {
    let inputstr = "(+ 1 (* 1 1 1) (- 3 2 1))";
    let tokens = tokenizer(inputstr.to_string());
    for t in tokens.iter() {
        println!("{:?}", t)
    }
}

#[derive(Debug,Clone)]
pub enum Token {
    OpenList,
    CloseList,
    Atom(String),
    Comment,
}

fn token_from(capture: Captures) -> Token {
    // select 2nd capture that lacks whitespace
    let c = capture.at(1).unwrap();

    if c.starts_with(';') {
        return Token::Comment;
    }

    match c {
        "(" | "[" => Token::OpenList,
        ")" | "]" => Token::CloseList,
        _ => Token::Atom(c.to_string()),
    }
}

#[derive(Debug)]
pub struct Reader {
    tokens: Vec<Token>,
    current_token: Option<Token>,
    position: usize,
}

impl Reader {
    fn new(tokens: Vec<Token>) -> Reader {
        let current = tokens.first().map(|t| t.clone());
        Reader {
            tokens: tokens,
            current_token: current,
            position: 0,
        }
    }

    fn peek(&self) -> Option<Token> {
        self.current_token.clone()
    }
}

#[test]
fn test_reader() {
    let inputstr = "(+ 1 (* 1 1 1) (- 3 2 1)) ;; abc \n (+ 1 2)";
    let tokens = tokenizer(inputstr.to_string());
    let reader = Reader::new(tokens);
    println!("{}", inputstr);
    for token in reader {
        println!("{:?}", token);
    }
}

impl Iterator for Reader {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.current_token.clone().and_then(|current| {
            self.position += 1;
            // change to vec::get() once we have moved to refs
            if self.position < self.tokens.len() {
                self.current_token = Some(self.tokens[self.position].clone())
            } else {
                self.current_token = None
            }
            current.into()
        })
    }
}

#[test]
fn test_read_form() {
    let inputstr = r#"~@(a b c)"#;
    let tokens = tokenizer(inputstr.to_string());
    let mut reader = Reader::new(tokens);
    let ast = read_form(&mut reader).unwrap();
    print!("{} => ", inputstr);
    println!("{}", ast);
}

macro_rules! macroexpand {
    ( $literal:expr, $reader:expr, $result:expr ) => {{
        let mut seq = vec![new_symbol($literal)];
        let _ = $reader.next();
        if let Ok(next) = read_form($reader) {
            seq.push(next);
            $result = Ok(new_list(seq));
        }
    }};
}

fn read_form(reader: &mut Reader) -> ReaderResult {
    let mut result: ReaderResult = Err(ReaderError::Message("could not read form".to_string()));

    while let Some(token) = reader.peek() {
        match token {
            Token::Atom(ref value) => {
                match value.as_str() {
                    "'" => macroexpand!("quote", reader, result),
                    "`" => macroexpand!("quasiquote", reader, result),
                    "~" => macroexpand!("unquote", reader, result),
                    "~@" => macroexpand!("splice-unquote", reader, result),
                    "@" => macroexpand!("deref", reader, result),
                    "" => {
                        result = Err(ReaderError::EmptyInput);
                        break;
                    }
                    _ => {
                        result = read_atom(reader);
                    }
                }
                break;
            }
            Token::OpenList => {
                result = read_list(reader);
                break;
            }
            Token::CloseList => break,
            Token::Comment => {
                let _ = reader.next();
            }
        }
    }
    result
}

fn read_list(reader: &mut Reader) -> ReaderResult {
    let _ = reader.next();
    let mut in_list = true;

    let mut list: Vec<LispValue> = vec![];

    while let Some(token) = reader.peek() {
        match token {
            Token::CloseList => {
                let _ = reader.next();
                in_list = false;
                break;
            }
            _ => {
                if let Ok(ast) = read_form(reader) {
                    list.push(ast);
                }
            }
        }
    }

    if in_list {
        return Err(ReaderError::Message("did not close list properly".to_string()));
    }

    Ok(new_list(list))
}

fn read_atom(reader: &mut Reader) -> ReaderResult {
    reader.next()
        .ok_or(ReaderError::Message("error missing tokens".to_string()))
        .and_then(|token| {
            match token {
                Token::Atom(ref s) => {
                    nil_from(s)
                        .or(boolean_from(s))
                        .or(number_from(s))
                        .or(keyword_from(s))
                        .or(string_from(s))
                        .or(symbol_from(s))
                }
                _ => {
                    Err(ReaderError::Message("reader: trying to get atom from non-atom token"
                        .to_string()))
                }
            }
        })
}

fn nil_from(token: &str) -> ReaderResult {
    match token {
            "nil" => Some(new_nil()),
            _ => None,
        }
        .ok_or(ReaderError::Message("could not parse nil".to_string()))
}

fn boolean_from(token: &str) -> ReaderResult {
    token.parse::<bool>()
        .map(new_boolean)
        .map_err(|_| ReaderError::Message("could not parse boolean from this token".to_string()))
}

fn number_from(token: &str) -> ReaderResult {
    token.parse::<i64>()
        .map(new_number)
        .map_err(|_| ReaderError::Message("could not parse number from this token".to_string()))
}

fn keyword_from(token: &str) -> ReaderResult {
    lazy_static! {
        static ref KEYWORD: Regex = Regex::new(r#"^:(.*)$"#).unwrap();
    }

    KEYWORD.captures(token)
        .and_then(|caps| caps.at(0))
        .ok_or(ReaderError::Message("could not parse keyword properly".to_string()))
        .and_then(|k| Ok(new_keyword(k)))
}

fn string_from(token: &str) -> ReaderResult {
    lazy_static! {
        static ref STRING: Regex = Regex::new(r#"^".*"$"#).unwrap();
    }

    if STRING.is_match(token) {
            let new_str = &token[1..token.len() - 1];
            Some(new_string(&read_str(new_str)))
        } else {
            None
        }
        .ok_or(ReaderError::Message("could not produce a string for this token".to_string()))
}


// NOTE: mal specifies the following:
// When a string is read, the following transformations are applied:
// a backslash followed by a doublequote is translated into a plain doublequote character
// a backslash followed by "n" is translated into a newline
// a backslash followed by another backslash is translated into a single backslash.
fn read_str(s: &str) -> String {
    s.replace(r#"\""#, "\"")
        .replace(r#"\n"#, "\n")
        .replace(r#"\\"#, "\\")
}

fn symbol_from(token: &str) -> ReaderResult {
    Ok(new_symbol(token))
}
