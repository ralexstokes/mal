#[derive(Debug,Clone)]
pub enum Ast {
    Symbol(String),
    Number(i64),
    List(Vec<Ast>),
    // Combination(PrimOpType, Box<Args>),
    PrimOp(PrimOpType),
}


#[derive(Debug,Clone)]
pub enum TokenType {
    OpenList,
    CloseList,
    Atom,
    Comment,
}

#[derive(Debug,Clone)]
pub enum PrimOpType {
    Add,
    Subtract,
    Multiply,
    Divide,
}
