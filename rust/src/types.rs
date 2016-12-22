#[derive(Debug)]
pub enum Ast {
    Symbol(String),
    Number(i64),
    List(Vec<Ast>),
}


#[derive(Debug,Clone)]
pub enum TokenType {
    OpenList,
    CloseList,
    Atom,
    Comment,
}
