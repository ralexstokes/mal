use regex::{Regex, Captures};
use types::{Ast, TokenType};

pub fn read_str(input: String) -> Option<Ast> {
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
    let tokens = tokenizer("(1 2, 3, 4,,,,)".to_string());
    for t in tokens.iter() {
        println!("{:?}", t)
    }
}

fn token_from(capture: Captures) -> Token {
    let c = capture.at(0).unwrap().trim_matches(is_whitespace);

    Token {
        typ: typ_for(c),
        value: c.to_string(),
    }
}

fn is_whitespace(c: char) -> bool {
    c == ',' || c.is_whitespace()
}

fn typ_for(c: &str) -> TokenType {
    if c.starts_with(';') {
        return TokenType::Comment;
    }

    match c {
        "(" => TokenType::OpenList,
        ")" => TokenType::CloseList,
        _ => TokenType::Atom,
    }
}

#[test]
fn test_read_form() {
    let inputstr = "(+ 1 2 (3 4))";
    let tokens = tokenizer(inputstr.to_string());
    let mut reader = Reader::new(tokens);
    let ast = read_form(&mut reader).unwrap();
    print!("{} => ", inputstr);
    println!("{:?}", ast);
}

fn read_form(reader: &mut Reader) -> Option<Ast> {
    match reader.peek() {
        Some(token) => ast_from(reader, &token),
        None => None,
    }
}

fn ast_from(reader: &mut Reader, token: &Token) -> Option<Ast> {
    match token.typ {
        TokenType::OpenList => read_list(reader),
        TokenType::Atom => read_atom(reader),
        _ => None,
    }
}

fn read_list(reader: &mut Reader) -> Option<Ast> {
    let mut list: Vec<Ast> = vec![];
    let mut did_close_list = false;

    while let Some(token) = reader.next() {
        match token.typ {
            TokenType::CloseList => {
                did_close_list = true;
                break;
            }
            _ => {
                if let Some(ast) = read_form(reader) {
                    list.push(ast)
                }
            }
        }
    }

    if !did_close_list {
        return None;
    }

    Some(Ast::List(list))
}

fn read_atom(reader: &mut Reader) -> Option<Ast> {
    if let Some(token) = reader.peek() {
        match token.typ {
            TokenType::Atom => {
                if let Some(node) = number_from(&token) {
                    return Some(node);
                }
                symbol_from(&token)
            }
            _ => None,
        }
    } else {
        None
    }
}

fn symbol_from(token: &Token) -> Option<Ast> {
    Some(Ast::Symbol(token.value.clone()))
}

fn number_from(token: &Token) -> Option<Ast> {
    match token.value.parse::<i64>() {
        Ok(n) => Some(Ast::Number(n)),
        Err(_) => None,
    }
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

impl Reader {
    fn new(tokens: Vec<Token>) -> Reader {
        let mut current: Option<Token> = None;

        if let Some(t) = tokens.first() {
            current = Some(t.clone());
        }

        Reader {
            tokens: tokens,
            current_token: current,
            position: 0,
        }
    }

    // next returns the token at the current position and increments the position.
    // returns None if we are past the end of the token stream.
    fn next(&mut self) -> Option<Token> {
        let token = self.current_token.clone();
        self.position += 1;
        if self.position < self.tokens.len() {
            self.current_token = Some(self.tokens[self.position].clone());
        } else {
            self.current_token = None
        }
        token
    }

    fn peek(&self) -> Option<Token> {
        self.current_token.clone()
    }
}


#[test]
fn test_reader() {
    let tokens = tokenizer("(+ 2 3)".to_string());
    let mut reader = Reader::new(tokens);
    while let Some(token) = reader.next() {
        println!("{:?}", token);
    }
}
