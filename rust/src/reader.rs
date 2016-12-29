use regex::{Regex, Captures};
use types::Ast;

pub fn read(input: String) -> Option<Ast> {
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
    let inputstr = "(+ 1 (* 1 1 1) (- 3 2 1))";
    let tokens = tokenizer(inputstr.to_string());
    let reader = Reader::new(tokens);
    println!("{}", inputstr);
    for token in reader {
        println!("{:?}", token);
    }
}

#[test]
fn test_read_form() {
    let inputstr = "(+ 1 2 (* 1 1 1) (- 3 2 1))";
    let tokens = tokenizer(inputstr.to_string());
    let mut reader = Reader::new(tokens);
    let ast = read_form(&mut reader).unwrap();
    print!("{} => ", inputstr);
    println!("{}", ast);
}

fn read_form(reader: &mut Reader) -> Option<Ast> {
    let mut result: Option<Ast> = None;

    while let Some(token) = reader.peek() {
        match token.typ {
            TokenType::Atom => {
                result = read_atom(reader);
                break;
            }
            TokenType::OpenList => {
                result = read_list(reader);
                break;
            }
            TokenType::CloseList => continue,
            TokenType::Comment => break,
        }
    }
    result
}

fn read_list(reader: &mut Reader) -> Option<Ast> {
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
                if let Some(ast) = read_form(reader) {
                    list.push(ast);
                }
            }
        }
    }

    if in_list {
        return None;
    }

    parse_list(&list).or(Some(Ast::Combination(list)))
}

const DO_FORM: &'static str = "do";
const IF_FORM: &'static str = "if";
const DEFINE_FORM: &'static str = "def!";
const LET_FORM: &'static str = "let*";
const FN_FORM: &'static str = "fn*";

fn parse_list(list: &Vec<Ast>) -> Option<Ast> {
    list.split_first()
        .and_then(|(first, rest)| {
            match *first {
                Ast::Symbol(ref s) => {
                    match s.as_str() {
                        DO_FORM => Some(Ast::Do(rest.to_vec())),
                        IF_FORM => make_if(rest.to_vec()),
                        DEFINE_FORM => make_define(rest.to_vec()),
                        LET_FORM => make_let(rest.to_vec()),
                        FN_FORM => make_lambda(rest.to_vec()),
                        &_ => None,
                    }
                }
                _ => None,
            }
        })
}

fn make_if(args: Vec<Ast>) -> Option<Ast> {
    match args.len() {
        2 => {
            Some(Ast::If {
                predicate: Box::new(args[0].clone()),
                consequent: Box::new(args[1].clone()),
                alternative: None,
            })
        }
        3 => {
            Some(Ast::If {
                predicate: Box::new(args[0].clone()),
                consequent: Box::new(args[1].clone()),
                alternative: Some(Box::new(args[2].clone())),
            })
        }
        _ => None,
    }
}

fn make_define(args: Vec<Ast>) -> Option<Ast> {
    args.split_first().and_then(|(key, vals)| {
        match *key {
            Ast::Symbol(ref s) => {
                vals.split_first().and_then(|(val, _)| {
                    Some(Ast::Define {
                        name: s.clone(),
                        val: Box::new(val.clone()),
                    })
                })
            }
            _ => None,
        }
    })
}

fn make_let(args: Vec<Ast>) -> Option<Ast> {
    args.split_first().and_then(|(bindings, body)| {
        match *bindings {
            Ast::Combination(ref seq) => {
                body.split_first().and_then(|(body, _)| {
                    Some(Ast::Let {
                        bindings: seq.to_vec(),
                        body: Box::new(body.clone()),
                    })
                })
            }
            _ => None,
        }
    })
}

fn make_lambda(args: Vec<Ast>) -> Option<Ast> {
    args.split_first().and_then(|(first, rest)| {
        match *first {
            Ast::Combination(ref seq) => {
                Some(Ast::Lambda {
                    bindings: seq.to_vec(),
                    body: Box::new(Ast::Do(rest.to_vec())),
                    env: None,
                })
            }
            _ => None,
        }
    })
}

fn read_atom(reader: &mut Reader) -> Option<Ast> {
    reader.next().and_then(|token| {
        nil_from(&token)
            .or(boolean_from(&token))
            .or(number_from(&token))
            .or(string_from(&token))
            .or(symbol_from(&token))
    })
}

fn nil_from(token: &Token) -> Option<Ast> {
    match token.value.as_str() {
        "nil" => Some(Ast::Nil),
        _ => None,
    }
}

fn boolean_from(token: &Token) -> Option<Ast> {
    match token.value.parse::<bool>() {
        Ok(p) => Some(Ast::Boolean(p)),
        Err(_) => None,
    }
}

fn number_from(token: &Token) -> Option<Ast> {
    match token.value.parse::<i64>() {
        Ok(n) => Some(Ast::Number(n)),
        Err(_) => None,
    }
}

fn string_from(token: &Token) -> Option<Ast> {
    let s = &token.value;

    if s.as_str().starts_with('"') {
        Some(Ast::String(s.clone()))
    } else {
        None
    }
}

fn symbol_from(token: &Token) -> Option<Ast> {
    Some(Ast::Symbol(token.value.clone()))
}
