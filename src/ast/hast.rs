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
use std::collections::HashSet;

use super::types::Type;

/// Table that contains all symbol and its types
pub type SymbolTable = HashMap<String, Option<Type>>;

/// High-level AST module - represents code as parsed from source files
/// This AST contains high-level constructs like match statements that don't
/// directly map to simple backend constructs
#[derive(Debug, Clone)]
pub struct HModule {
    pub imports: HashSet<String>,
    pub func: Vec<HFunction>,
    pub structs: Vec<HStructDef>,
    pub globals: Vec<String>,
}

impl HModule {
    pub fn merge_with(&mut self, mut other: HModule) {
        self.func.append(&mut other.func);
        self.structs.append(&mut other.structs);
        self.globals.append(&mut other.globals)
    }

    pub fn get_symbol_table(&self) -> SymbolTable {
        let mut table = SymbolTable::new();

        for func in self.func.clone() {
            table.insert(func.name, func.ret_type);
        }

        table
    }
}

#[derive(Debug, Clone)]
pub struct HFunction {
    pub name: String,
    pub arguments: Vec<HVariable>,
    pub body: HStatement,
    pub ret_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct HStructDef {
    pub name: String,
    pub fields: Vec<HVariable>,
    pub methods: Vec<HFunction>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct HVariable {
    pub name: String,
    pub ty: Option<Type>,
}

impl AsRef<HVariable> for HVariable {
    fn as_ref(&self) -> &Self {
        self
    }
}

/// High-level statements include constructs like match that will be
/// lowered to simpler constructs in the LLAST
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum HStatement {
    /// (Statements, Scoped variables)
    Block {
        statements: Vec<HStatement>,
        scope: Vec<HVariable>,
    },
    Declare {
        variable: HVariable,
        value: Option<HExpression>,
    },
    Assign {
        lhs: Box<HExpression>,
        rhs: Box<HExpression>,
    },
    Return(Option<HExpression>),
    If {
        condition: HExpression,
        body: Box<HStatement>,
        else_branch: Option<Box<HStatement>>,
    },
    While {
        condition: HExpression,
        body: Box<HStatement>,
    },
    For {
        ident: HVariable,
        expr: HExpression,
        body: Box<HStatement>,
    },
    /// High-level match statement that will be lowered to switch/if-else
    Match {
        subject: HExpression,
        arms: Vec<HMatchArm>,
    },
    Break,
    Continue,
    Exp(HExpression),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum HExpression {
    Int(usize),
    Str(String),
    Bool(bool),
    /// Represents "self" keyword
    Selff,
    Array {
        capacity: usize,
        elements: Vec<HExpression>,
    },
    FunctionCall {
        fn_name: String,
        args: Vec<HExpression>,
    },
    Variable(String),
    ArrayAccess {
        name: String,
        index: Box<HExpression>,
    },
    BinOp {
        lhs: Box<HExpression>,
        op: HBinOp,
        rhs: Box<HExpression>,
    },
    StructInitialization {
        name: String,
        fields: HashMap<String, Box<HExpression>>,
    },
    FieldAccess {
        expr: Box<HExpression>,
        field: Box<HExpression>,
    },
}

impl TryFrom<Token> for HExpression {
    type Error = String;

    fn try_from(token: Token) -> std::result::Result<Self, String> {
        let kind = token.kind;
        match kind {
            TokenKind::Identifier(val) => Ok(HExpression::Variable(val)),
            TokenKind::Literal(Value::Int) => Ok(HExpression::Int(
                token
                    .raw
                    .parse()
                    .map_err(|_| "Int value could not be parsed")?,
            )),
            TokenKind::Keyword(Keyword::Boolean) => match token.raw.as_ref() {
                "true" => Ok(HExpression::Bool(true)),
                "false" => Ok(HExpression::Bool(false)),
                _ => Err("Boolean value could not be parsed".into()),
            },
            TokenKind::Literal(Value::Str(string)) => Ok(HExpression::Str(string)),
            _ => Err("Value could not be parsed".into()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum HMatchArm {
    Case(HExpression, HStatement),
    Else(HStatement),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum HBinOp {
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

impl TryFrom<TokenKind> for HBinOp {
    type Error = String;
    fn try_from(token: TokenKind) -> Result<HBinOp, String> {
        match token {
            TokenKind::Star => Ok(HBinOp::Multiplication),
            TokenKind::Slash => Ok(HBinOp::Division),
            TokenKind::Plus => Ok(HBinOp::Addition),
            TokenKind::Minus => Ok(HBinOp::Subtraction),
            TokenKind::Percent => Ok(HBinOp::Modulus),
            TokenKind::LessThan => Ok(HBinOp::LessThan),
            TokenKind::GreaterThan => Ok(HBinOp::GreaterThan),
            TokenKind::Equals => Ok(HBinOp::Equal),
            TokenKind::LessThanOrEqual => Ok(HBinOp::LessThanOrEqual),
            TokenKind::GreaterThanOrEqual => Ok(HBinOp::GreaterThanOrEqual),
            TokenKind::NotEqual => Ok(HBinOp::NotEqual),
            TokenKind::And => Ok(HBinOp::And),
            TokenKind::Or => Ok(HBinOp::Or),
            TokenKind::PlusEqual => Ok(HBinOp::AddAssign),
            TokenKind::MinusEqual => Ok(HBinOp::SubtractAssign),
            TokenKind::StarEqual => Ok(HBinOp::MultiplyAssign),
            TokenKind::SlashEqual => Ok(HBinOp::DivideAssign),
            other => Err(format!(
                "Token {:?} cannot be converted into a HBinOp",
                other
            )),
        }
    }
}