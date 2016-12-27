#[derive(Debug,Clone)]
pub enum TokenType {
    OpenList,
    CloseList,
    Atom,
    Comment,
    String,
}

#[derive(Debug,Clone)]
pub enum Primitive {
    Add,
    Subtract,
    Multiply,
    Divide,

    Define,
    Let,
}

#[derive(Debug,Clone)]
pub enum Ast {
    Nil,
    True,
    False,
    Symbol(String),
    String(String),
    Number(i64),
    List(Vec<Ast>),
}
