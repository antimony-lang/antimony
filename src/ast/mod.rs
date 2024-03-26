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
use crate::lexer::{Keyword, Token, TokenKind, Value};
use core::convert::TryFrom;
use std::collections::HashMap;
use std::collections::HashSet;

pub mod types;
use types::Type;

#[derive(Debug, Clone)]
pub struct Module {
    pub imports: HashSet<String>,
    pub func: Vec<Function>,
    pub structs: Vec<StructDef>,
    pub globals: Vec<String>,
}

impl Module {
    pub fn merge_with(&mut self, mut other: Module) {
        self.func.append(&mut other.func);
        self.structs.append(&mut other.structs);
        self.globals.append(&mut other.globals)
    }
}

#[derive(Debug, Clone)]
pub struct Callable {
    pub name: String,
    pub arguments: Vec<TypedVariable>,
    pub ret_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub callable: Callable,
    pub body: Option<Statement>,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<TypedVariable>,
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone)]
pub struct Method {
    pub callable: Callable,
    pub body: Statement,
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

impl From<TypedVariable> for Variable {
    fn from(typed: TypedVariable) -> Self {
        Self {
            name: typed.name,
            ty: Some(typed.ty),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TypedVariable {
    pub name: String,
    pub ty: Type,
}

impl AsRef<TypedVariable> for TypedVariable {
    fn as_ref(&self) -> &Self {
        self
    }
}

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
        op: AssignOp,
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
    Match {
        subject: Expression,
        arms: Vec<MatchArm>,
    },
    Break,
    Continue,
    Exp(Expression),
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AssignOp {
    /// '='
    Set,
    /// '+='
    Add,
    /// '-='
    Subtract,
    /// '*='
    Multiply,
    /// '/='
    Divide,
}

impl TryFrom<TokenKind> for AssignOp {
    type Error = String;
    fn try_from(token: TokenKind) -> Result<AssignOp, String> {
        match token {
            TokenKind::Assign => Ok(AssignOp::Set),
            TokenKind::PlusEqual => Ok(AssignOp::Add),
            TokenKind::MinusEqual => Ok(AssignOp::Subtract),
            TokenKind::StarEqual => Ok(AssignOp::Multiply),
            TokenKind::SlashEqual => Ok(AssignOp::Divide),
            other => Err(format!(
                "Token {:?} cannot be converted into an AssignOp",
                other
            )),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Expression {
    Int(usize),
    Str(String),
    Bool(bool),
    /// Represents "self" keyword
    Selff,
    Array(Vec<Expression>),
    FunctionCall {
        expr: Box<Expression>,
        args: Vec<Expression>,
    },
    Variable(String),
    ArrayAccess {
        expr: Box<Expression>,
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
        field: String,
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
pub enum MatchArm {
    Case(Expression, Statement),
    Else(Statement),
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
            other => Err(format!(
                "Token {:?} cannot be converted into a BinOp",
                other
            )),
        }
    }
}
