//! TSL Abstract Syntax Tree

/// Complete TSL program
#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

/// TSL types
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Trit,
    Tryte,
    Word,
    Array(Box<Type>, usize),
}

/// Statement types
#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        ty: Option<Type>,
        value: Expression,
    },
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    Return(Option<Expression>),
    Expression(Expression),
}

/// Expression types
#[derive(Debug, Clone)]
pub enum Expression {
    TritLiteral(i8),
    IntLiteral(i64),
    StringLiteral(String),
    Ident(String),
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    Call {
        name: String,
        args: Vec<Expression>,
    },
    Index {
        array: Box<Expression>,
        index: Box<Expression>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Xor,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Not,
    Rotate,
    RotateInv,
}
