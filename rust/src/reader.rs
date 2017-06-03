use regex::{Regex, Captures};
use types::{LispValue, Seq, new_symbol, new_list, new_nil, new_boolean, new_number, new_string,
            new_keyword, new_vector, new_map_from_seq};
use error::{ReaderError, EvaluationError};

pub type ReaderResult = ::std::result::Result<LispValue, ReaderError>;

pub fn read(input: String) -> ReaderResult {
    let tokens = tokenizer(input);
    let mut reader = Reader::new(tokens);
    let result = read_form(&mut reader);
    if reader.is_empty() {
        result
    } else {
        let (tokens, pos) = reader.extra_input();
        Err(ReaderError::ExtraInput(tokens, pos))
    }
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
    List(Direction),
    Vector(Direction),
    Map(Direction),
    Atom(String),
    Comment(String),
}

// Direction indicates if the wrapping token is opening or closing its extent.
#[derive(Debug,Clone)]
pub enum Direction {
    Open,
    Close,
}

fn token_from(capture: Captures) -> Token {
    // select 2nd capture that lacks whitespace
    let c = capture.at(1).unwrap();

    if c.starts_with(';') {
        return Token::Comment(c.to_string());
    }

    match c {
        "(" => Token::List(Direction::Open),
        ")" => Token::List(Direction::Close),
        "[" => Token::Vector(Direction::Open),
        "]" => Token::Vector(Direction::Close),
        "{" => Token::Map(Direction::Open),
        "}" => Token::Map(Direction::Close),
        _ => Token::Atom(c.to_string()),
    }
}

#[derive(Debug)]
struct Reader {
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

    fn extra_input(&self) -> (Vec<Token>, usize) {
        (self.tokens.clone(), self.position)
    }

    fn is_empty(&self) -> bool {
        self.position == self.tokens.len()
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
        let mut seq = vec![new_symbol($literal, None)];
        let _ = $reader.next();
        if let Ok(next) = read_form($reader) {
            seq.push(next);
            $result = Ok(new_list(seq, None));
        }
    }};
}

fn read_form(reader: &mut Reader) -> ReaderResult {
    let mut result: ReaderResult = Err(ReaderError::Message("could not read form".to_string()));

    while let Some(token) = reader.peek() {
        match token {
            Token::Comment(_) => {
                let _ = reader.next();
            }
            Token::Atom(ref value) => {
                match value.as_str() {
                    "'" => macroexpand!("quote", reader, result),
                    "`" => macroexpand!("quasiquote", reader, result),
                    "~" => macroexpand!("unquote", reader, result),
                    "~@" => macroexpand!("splice-unquote", reader, result),
                    "@" => macroexpand!("deref", reader, result),
                    "^" => {
                        let _ = reader.next();
                        result = read_form(reader).and_then(|meta| {
                            read_form(reader).and_then(|data| {
                                Ok(new_list(vec![new_symbol("with-meta", None), data, meta], None))
                            })
                        });
                    }
                    "" => {
                        result = Err(ReaderError::EmptyInput);
                    }
                    _ => {
                        result = read_atom(reader);
                    }
                }
                break;
            }
            Token::List(dir) => {
                match dir {
                    Direction::Open => {
                        result = read_list(reader);
                        break;
                    }
                    Direction::Close => break,
                }
            }
            Token::Vector(dir) => {
                match dir {
                    Direction::Open => {
                        result = read_vector(reader);
                        break;
                    }
                    Direction::Close => break,
                }
            }
            Token::Map(dir) => {
                match dir {
                    Direction::Open => {
                        result = read_map(reader);
                        break;
                    }
                    Direction::Close => break,
                }
            }
        }
    }
    result
}

fn read_seq(reader: &mut Reader) -> Result<Seq, ReaderError> {
    let _ = reader.next();
    let mut in_seq = true;

    let mut seq: Vec<LispValue> = vec![];

    while let Some(token) = reader.peek() {
        match token {
            Token::List(Direction::Close) |
            Token::Vector(Direction::Close) |
            Token::Map(Direction::Close) => {
                let _ = reader.next();
                in_seq = false;
                break;
            }
            _ => {
                if let Ok(form) = read_form(reader) {
                    seq.push(form);
                }
            }
        }
    }

    if in_seq {
        return Err(ReaderError::Message("did not close seq properly".to_string()));
    }

    Ok(seq)
}

fn read_list(reader: &mut Reader) -> ReaderResult {
    read_seq(reader).and_then(|seq| Ok(new_list(seq, None)))
}

fn read_vector(reader: &mut Reader) -> ReaderResult {
    read_seq(reader).and_then(|seq| Ok(new_vector(seq, None)))
}

fn read_map(reader: &mut Reader) -> ReaderResult {
    read_seq(reader).and_then(|seq| {
        match new_map_from_seq(seq) {
            Ok(map) => Ok(map),
            Err(EvaluationError::Message(ref s)) => Err(ReaderError::Message(s.clone())),
            _ => Err(ReaderError::Message("could not read map".to_string())),
        }
    })
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
                        .or(symbol_from(s)) // catch-all, call last
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
    // NOTE: Want to only store the suffix, caps.at(1)
    // However, it is pretty straightforward to hack around
    // defects in the current map impl if we keep it for now
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

// symbol_from will accept any token so it needs to be called as a catch-all
fn symbol_from(token: &str) -> ReaderResult {
    Ok(new_symbol(token, None))
}
