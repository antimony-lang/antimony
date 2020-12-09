use crate::parser::{Token, TokenKind, Value};
use core::convert::TryFrom;

#[derive(Debug)]
pub struct Program {
    pub func: Vec<Function>,
    pub globals: Vec<String>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub arguments: Vec<Variable>,
    pub body: Statement,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Statement {
    Block(Vec<Statement>),
    Declare(Variable, Option<Expression>),
    Return(Option<Expression>),
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    While(Expression, Box<Statement>),
    Exp(Expression),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Expression {
    Int(u32),
    Str(String),
    Char(u8),
    FunctionCall(String, Vec<Expression>),
    Variable(String),
    Assign(String, Box<Expression>),
    BinOp(Box<Expression>, BinOp, Box<Expression>),
}

impl TryFrom<Token> for Expression {
    type Error = String;

    fn try_from(token: Token) -> std::result::Result<Self, String> {
        let kind = token.kind;
        match kind {
            TokenKind::Identifier(val) => Ok(Expression::Variable(val)),
            TokenKind::Literal(Value::Int) => Ok(Expression::Int(
                token
                    .raw
                    .parse()
                    .map_err(|_| "Int value could not be parsed")?,
            )),
            TokenKind::Literal(Value::Str) => Ok(Expression::Str(token.raw)),
            other => panic!("Value could not be parsed"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BinOp {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulus,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
    And,
    Or,
}

impl TryFrom<TokenKind> for BinOp {
    type Error = String;
    fn try_from(token: TokenKind) -> Result<BinOp, String> {
        match token {
            TokenKind::Star => Ok(BinOp::Multiplication),
            TokenKind::Slash => Ok(BinOp::Division),
            TokenKind::Plus => Ok(BinOp::Addition),
            TokenKind::Minus => Ok(BinOp::Subtraction),
            TokenKind::LessThan => Ok(BinOp::LessThan),
            TokenKind::GreaterThan => Ok(BinOp::GreaterThan),
            TokenKind::Equals => Ok(BinOp::Equal),
            TokenKind::LessThanOrEqual => Ok(BinOp::LessThanOrEqual),
            TokenKind::GreaterThanOrEqual => Ok(BinOp::GreaterThanOrEqual),
            TokenKind::NotEqual => Ok(BinOp::NotEqual),
            // TokenKind::And => BinOp::And,
            // TokenKind::Or => BinOp::Or,
            other => Err(format!("Token {:?} cannot be converted into a BinOp", other).into()),
        }
    }
}
