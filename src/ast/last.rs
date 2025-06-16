/**
 * Copyright 2021 Garrit Franke
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
use std::collections::HashMap;

use super::types::Type;

/// Table that contains all symbol and its types
pub type SymbolTable = HashMap<String, Option<Type>>;

/// Low-level AST module - represents code ready for code generation
/// This AST contains only simple constructs that map directly to backend targets
/// High-level constructs like match statements are lowered to if-else chains
#[derive(Debug, Clone)]
pub struct Module {
    pub func: Vec<Function>,
    pub structs: Vec<StructDef>,
    pub globals: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arguments: Vec<Variable>,
    pub body: Statement,
    pub ret_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Variable>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Variable {
    pub name: String,
    pub ty: Option<Type>,
}

impl AsRef<Variable> for Variable {
    fn as_ref(&self) -> &Self {
        self
    }
}

/// Low-level statements contain only simple constructs that map directly
/// to backend code generation. Complex constructs like match are lowered
/// to simpler if-else chains
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Statement {
    /// (Statements, Scoped variables)
    Block {
        statements: Vec<Statement>,
        scope: Vec<Variable>,
    },
    Declare {
        variable: Variable,
        value: Option<Expression>,
    },
    Assign {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Return(Option<Expression>),
    If {
        condition: Expression,
        body: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    For {
        ident: Variable,
        expr: Expression,
        body: Box<Statement>,
    },
    Break,
    Continue,
    Exp(Expression),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Expression {
    Int(usize),
    Str(String),
    Bool(bool),
    /// Represents "self" keyword
    Selff,
    Array {
        capacity: usize,
        elements: Vec<Expression>,
    },
    FunctionCall {
        fn_name: String,
        args: Vec<Expression>,
    },
    Variable(String),
    ArrayAccess {
        name: String,
        index: Box<Expression>,
    },
    BinOp {
        lhs: Box<Expression>,
        op: BinOp,
        rhs: Box<Expression>,
    },
    StructInitialization {
        name: String,
        fields: HashMap<String, Box<Expression>>,
    },
    FieldAccess {
        expr: Box<Expression>,
        field: Box<Expression>,
    },
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
            TokenKind::Literal(Value::Str(string)) => Ok(Expression::Str(string)),
            _ => Err("Value could not be parsed".into()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
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
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
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
            TokenKind::PlusEqual => Ok(BinOp::AddAssign),
            TokenKind::MinusEqual => Ok(BinOp::SubtractAssign),
            TokenKind::StarEqual => Ok(BinOp::MultiplyAssign),
            TokenKind::SlashEqual => Ok(BinOp::DivideAssign),
            other => Err(format!(
                "Token {:?} cannot be converted into a BinOp",
                other
            )),
        }
    }
}
