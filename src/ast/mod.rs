use crate::lexer::*;
use core::convert::TryFrom;
use indextree::Arena;
use std::collections::HashMap;
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
use std::collections::HashSet;

pub mod types;
use types::Type;

#[derive(Debug, Clone)]
pub enum ASTNode {
    ModuleNode(Module),
    FunctionNode(Function),
    StructDefNode(StructDef),
    VariableNode(Variable),
    StatementNode(Statement),
    ExpressionNode(Expression),
    MatchArmNode(MatchArm),
    BinOpNode(BinOp),
}

/// Table that contains all symbol and its types
pub type SymbolTable = HashMap<String, Option<Type>>;

#[derive(Debug, Clone)]
pub struct Module {
    pub path: String,
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

    pub fn get_symbol_table(&self) -> SymbolTable {
        let mut table = SymbolTable::new();

        for func in self.func.clone() {
            table.insert(func.name, func.ret_type);
        }

        table
    }
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

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Statement {
    /// (Statements, Scoped variables)
    Block(Vec<Statement>, Vec<Variable>),
    Declare(Variable, Option<Expression>),
    Assign(Box<Expression>, Box<Expression>),
    Return(Option<Expression>),
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    While(Expression, Box<Statement>),
    For(Variable, Expression, Box<Statement>),
    Match(Expression, Vec<MatchArm>),
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
    Array(usize, Vec<Expression>),
    FunctionCall(String, Vec<Expression>),
    Variable(String),
    /// (name, index)
    ArrayAccess(String, Box<Expression>),
    BinOp(Box<Expression>, BinOp, Box<Expression>),
    StructInitialization(String, HashMap<String, Box<Expression>>),
    FieldAccess(Box<Expression>, Box<Expression>),
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
