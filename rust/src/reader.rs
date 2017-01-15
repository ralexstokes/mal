use regex::{Regex, Captures};
use types::Ast;
use error::ReaderError;

pub type ReaderResult = ::std::result::Result<Ast, ReaderError>;

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

fn token_from(capture: Captures) -> Token {
    // select 2nd capture that lacks whitespace
    let c = capture.at(1).unwrap();

    Token {
        typ: typ_for(c),
        value: c.to_string(),
    }
}

fn typ_for(c: &str) -> TokenType {
    if c.starts_with(';') {
        return TokenType::Comment;
    }

    match c {
        "(" | "[" => TokenType::OpenList,
        ")" | "]" => TokenType::CloseList,
        _ => TokenType::Atom,
    }
}

#[derive(Debug,Clone)]
pub enum TokenType {
    OpenList,
    CloseList,
    Atom,
    Comment,
}


#[derive(Debug,Clone)]
pub struct Token {
    typ: TokenType,
    value: String,
}

#[derive(Debug)]
pub struct Reader {
    tokens: Vec<Token>,
    current_token: Option<Token>,
    position: usize,
}

impl Iterator for Reader {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let current_token = match self.current_token {
            Some(_) => self.current_token.clone(),
            None => return None,
        };

        self.position += 1;
        if self.position < self.tokens.len() {
            self.current_token = Some(self.tokens[self.position].clone())
        } else {
            self.current_token = None
        }
        current_token
    }
}

impl Reader {
    fn new(tokens: Vec<Token>) -> Reader {
        let current = tokens.first()
            .map(|t| t.clone());

        match current {
            Some(_) => {
                Reader {
                    tokens: tokens,
                    current_token: current,
                    position: 0,
                }
            }
            None => {
                Reader {
                    tokens: tokens,
                    current_token: None,
                    position: 0,
                }
            }
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

#[test]
fn test_read_form() {
    let inputstr = r#"~@(a b c)"#;
    let tokens = tokenizer(inputstr.to_string());
    let mut reader = Reader::new(tokens);
    let ast = read_form(&mut reader).unwrap();
    print!("{} => ", inputstr);
    println!("{}", ast);
}

fn read_form(reader: &mut Reader) -> ReaderResult {
    let mut result: ReaderResult = Err(ReaderError::Message("could not read form".to_string()));

    while let Some(token) = reader.peek() {
        match token.typ {
            TokenType::Atom => {
                match token.value.as_str() {
                    "'" => {
                        let mut seq = vec![Ast::Symbol("quote".to_string())];
                        let _ = reader.next();
                        if let Ok(next) = read_form(reader) {
                            seq.push(next);
                            result = Ok(Ast::List(seq));
                        }
                    }
                    "`" => {
                        let mut seq = vec![Ast::Symbol("quasiquote".to_string())];
                        let _ = reader.next();
                        if let Ok(next) = read_form(reader) {
                            seq.push(next);
                            result = Ok(Ast::List(seq))
                        }
                    }
                    "~" => {
                        let mut seq = vec![Ast::Symbol("unquote".to_string())];
                        let _ = reader.next();
                        if let Ok(next) = read_form(reader) {
                            seq.push(next);
                            result = Ok(Ast::List(seq))
                        }
                    }
                    "~@" => {
                        let mut seq = vec![Ast::Symbol("splice-unquote".to_string())];
                        let _ = reader.next();
                        if let Ok(next) = read_form(reader) {
                            seq.push(next);
                            result = Ok(Ast::List(seq))
                        }
                    }
                    _ => {
                        result = read_atom(reader);
                    }
                }
                break;
            }
            TokenType::OpenList => {
                result = read_list(reader);
                break;
            }
            TokenType::CloseList => break,
            TokenType::Comment => {
                let _ = reader.next();
            }
        }
    }
    result
}

fn read_list(reader: &mut Reader) -> ReaderResult {
    let _ = reader.next();
    let mut in_list = true;

    let mut list: Vec<Ast> = vec![];

    while let Some(token) = reader.peek() {
        match token.typ {
            TokenType::CloseList => {
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

    Ok(Ast::List(list))
}

fn read_atom(reader: &mut Reader) -> ReaderResult {
    reader.next()
        .ok_or(ReaderError::Message("error missing tokens".to_string()))
        .and_then(|token| {
            nil_from(&token)
                .or(boolean_from(&token))
                .or(number_from(&token))
                .or(string_from(&token))
                .or(symbol_from(&token))
        })
}

fn nil_from(token: &Token) -> ReaderResult {
    match token.value.as_str() {
            "nil" => Some(Ast::Nil),
            _ => None,
        }
        .ok_or(ReaderError::Message("could not parse nil".to_string()))
}

fn boolean_from(token: &Token) -> ReaderResult {
    token.value
        .parse::<bool>()
        .map(|p| Ast::Boolean(p))
        .map_err(|_| ReaderError::Message("could not parse boolean from this token".to_string()))
}

fn number_from(token: &Token) -> ReaderResult {
    token.value
        .parse::<i64>()
        .map(|n| Ast::Number(n))
        .map_err(|_| ReaderError::Message("could not parse number from this token".to_string()))
}

fn string_from(token: &Token) -> ReaderResult {
    let s = &token.value;

    lazy_static! {
        static ref STRING: Regex = Regex::new(r#"^".*"$"#).unwrap();
    }

    if STRING.is_match(s) {
            let new_str = &s[1..s.len() - 1];
            Some(Ast::String(read_str(new_str)))
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

fn symbol_from(token: &Token) -> ReaderResult {
    Ok(Ast::Symbol(token.value.clone()))
}
