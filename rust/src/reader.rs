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
        "@" => TokenType::Sigil(SigilType::AtomDeref),
        _ => TokenType::Atom,
    }
}

#[derive(Debug,Clone)]
enum TokenType {
    OpenList,
    CloseList,
    Atom,
    Comment,
    Sigil(SigilType),
}

#[derive(Debug,Clone)]
enum SigilType {
    AtomDeref,
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
    // let inputstr = "(+ 1 2 (* 1 1 1) (- 3 2 1) ;; abc \n (+ 1 2))";
    let inputstr = r#"(do ;; A comment in a file
(def! inc4 (fn* (a) (+ 4 a)))
(def! inc5 (fn* (a)  ;; a comment after code
  (+ 5 a)))

(prn "incB.mal finished")
"incB.mal return string"

;; ending comment

)"#;
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
            TokenType::CloseList => break, // Err(::UnexpectedInput)
            TokenType::Comment => {
                let _ = reader.next();
            }
            TokenType::Sigil(sigil) => {
                let _ = reader.next();
                result = match sigil {
                    SigilType::AtomDeref => {
                        read_form(reader).and_then(|next| {
                            let elems = vec![Ast::Symbol("deref".to_string()), next];
                            Ast::List(elems).into()
                        })
                    }
                };
                break;
            }
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

    Some(Ast::List(list))
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

    lazy_static! {
        static ref STRING: Regex = Regex::new(r#"^".*"$"#).unwrap();
    }

    if STRING.is_match(s) {
        let new_str = &s[1..s.len() - 1];
        Some(Ast::String(read_str(new_str)))
    } else {
        None
    }
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

fn symbol_from(token: &Token) -> Option<Ast> {
    Some(Ast::Symbol(token.value.clone()))
}
