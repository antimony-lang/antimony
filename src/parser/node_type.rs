/**
 * Copyright 2020 Garrit Franke
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use crate::lexer::*;
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
    pub ret_type: Option<Type>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Variable {
    pub name: String,
    pub ty: Option<Type>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Type {
    Int,
    Str,
    Bool,
    Array(Box<Type>),
}

impl TryFrom<String> for Type {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_ref() {
            "int" => Ok(Self::Int),
            "string" => Ok(Self::Str),
            "bool" => Ok(Self::Bool),
            _ => Err("Expected Type".into()),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Statement {
    Block(Vec<Statement>),
    Declare(Variable, Option<Expression>),
    Assign(Box<Expression>, Box<Expression>),
    Return(Option<Expression>),
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    While(Expression, Box<Statement>),
    Exp(Expression),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Expression {
    Int(u32),
    Str(String),
    Bool(bool),
    Array(Vec<Expression>),
    FunctionCall(String, Vec<Expression>),
    Variable(String),
    /// (name, index)
    ArrayAccess(String, Box<Expression>),
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
            TokenKind::Keyword(Keyword::Boolean) => match token.raw.as_ref() {
                "true" => Ok(Expression::Bool(true)),
                "false" => Ok(Expression::Bool(false)),
                _ => Err("Boolean value could not be parsed".into()),
            },
            TokenKind::Literal(Value::Str) => Ok(Expression::Str(token.raw)),
            _ => Err("Value could not be parsed".into()),
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
            TokenKind::Percent => Ok(BinOp::Modulus),
            TokenKind::LessThan => Ok(BinOp::LessThan),
            TokenKind::GreaterThan => Ok(BinOp::GreaterThan),
            TokenKind::Equals => Ok(BinOp::Equal),
            TokenKind::LessThanOrEqual => Ok(BinOp::LessThanOrEqual),
            TokenKind::GreaterThanOrEqual => Ok(BinOp::GreaterThanOrEqual),
            TokenKind::NotEqual => Ok(BinOp::NotEqual),
            TokenKind::And => Ok(BinOp::And),
            TokenKind::Or => Ok(BinOp::Or),
            other => Err(format!("Token {:?} cannot be converted into a BinOp", other).into()),
        }
    }
}
