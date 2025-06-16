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
use super::{Module, Function, StructDef, Variable, Statement, Expression, MatchArm, BinOp};

/// Low-level AST module - represents code ready for code generation
/// This AST contains only simple constructs that map directly to backend targets
/// High-level constructs like match statements are lowered to if-else chains
#[derive(Debug, Clone)]
pub struct LModule {
    pub imports: HashSet<String>,
    pub func: Vec<LFunction>,
    pub structs: Vec<LStructDef>,
    pub globals: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LFunction {
    pub name: String,
    pub arguments: Vec<LVariable>,
    pub body: LStatement,
    pub ret_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct LStructDef {
    pub name: String,
    pub fields: Vec<LVariable>,
    pub methods: Vec<LFunction>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct LVariable {
    pub name: String,
    pub ty: Option<Type>,
}

impl AsRef<LVariable> for LVariable {
    fn as_ref(&self) -> &Self {
        self
    }
}

/// Low-level statements contain only simple constructs that map directly
/// to backend code generation. Complex constructs like match are lowered
/// to simpler if-else chains
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LStatement {
    /// (Statements, Scoped variables)
    Block {
        statements: Vec<LStatement>,
        scope: Vec<LVariable>,
    },
    Declare {
        variable: LVariable,
        value: Option<LExpression>,
    },
    Assign {
        lhs: Box<LExpression>,
        rhs: Box<LExpression>,
    },
    Return(Option<LExpression>),
    If {
        condition: LExpression,
        body: Box<LStatement>,
        else_branch: Option<Box<LStatement>>,
    },
    While {
        condition: LExpression,
        body: Box<LStatement>,
    },
    For {
        ident: LVariable,
        expr: LExpression,
        body: Box<LStatement>,
    },
    Break,
    Continue,
    Exp(LExpression),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LExpression {
    Int(usize),
    Str(String),
    Bool(bool),
    /// Represents "self" keyword
    Selff,
    Array {
        capacity: usize,
        elements: Vec<LExpression>,
    },
    FunctionCall {
        fn_name: String,
        args: Vec<LExpression>,
    },
    Variable(String),
    ArrayAccess {
        name: String,
        index: Box<LExpression>,
    },
    BinOp {
        lhs: Box<LExpression>,
        op: LBinOp,
        rhs: Box<LExpression>,
    },
    StructInitialization {
        name: String,
        fields: HashMap<String, Box<LExpression>>,
    },
    FieldAccess {
        expr: Box<LExpression>,
        field: Box<LExpression>,
    },
}

impl TryFrom<Token> for LExpression {
    type Error = String;

    fn try_from(token: Token) -> std::result::Result<Self, String> {
        let kind = token.kind;
        match kind {
            TokenKind::Identifier(val) => Ok(LExpression::Variable(val)),
            TokenKind::Literal(Value::Int) => Ok(LExpression::Int(
                token
                    .raw
                    .parse()
                    .map_err(|_| "Int value could not be parsed")?,
            )),
            TokenKind::Keyword(Keyword::Boolean) => match token.raw.as_ref() {
                "true" => Ok(LExpression::Bool(true)),
                "false" => Ok(LExpression::Bool(false)),
                _ => Err("Boolean value could not be parsed".into()),
            },
            TokenKind::Literal(Value::Str(string)) => Ok(LExpression::Str(string)),
            _ => Err("Value could not be parsed".into()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LBinOp {
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

impl TryFrom<TokenKind> for LBinOp {
    type Error = String;
    fn try_from(token: TokenKind) -> Result<LBinOp, String> {
        match token {
            TokenKind::Star => Ok(LBinOp::Multiplication),
            TokenKind::Slash => Ok(LBinOp::Division),
            TokenKind::Plus => Ok(LBinOp::Addition),
            TokenKind::Minus => Ok(LBinOp::Subtraction),
            TokenKind::Percent => Ok(LBinOp::Modulus),
            TokenKind::LessThan => Ok(LBinOp::LessThan),
            TokenKind::GreaterThan => Ok(LBinOp::GreaterThan),
            TokenKind::Equals => Ok(LBinOp::Equal),
            TokenKind::LessThanOrEqual => Ok(LBinOp::LessThanOrEqual),
            TokenKind::GreaterThanOrEqual => Ok(LBinOp::GreaterThanOrEqual),
            TokenKind::NotEqual => Ok(LBinOp::NotEqual),
            TokenKind::And => Ok(LBinOp::And),
            TokenKind::Or => Ok(LBinOp::Or),
            TokenKind::PlusEqual => Ok(LBinOp::AddAssign),
            TokenKind::MinusEqual => Ok(LBinOp::SubtractAssign),
            TokenKind::StarEqual => Ok(LBinOp::MultiplyAssign),
            TokenKind::SlashEqual => Ok(LBinOp::DivideAssign),
            other => Err(format!(
                "Token {:?} cannot be converted into a LBinOp",
                other
            )),
        }
    }
}

// Conversion implementations from LLAST to old AST for backward compatibility with generators
impl From<LModule> for Module {
    fn from(lmodule: LModule) -> Self {
        let func = lmodule.func.into_iter().map(Function::from).collect();
        let structs = lmodule.structs.into_iter().map(StructDef::from).collect();
        
        Module {
            imports: lmodule.imports,
            func,
            structs,
            globals: lmodule.globals,
        }
    }
}

impl From<LFunction> for Function {
    fn from(lfunc: LFunction) -> Self {
        let arguments = lfunc.arguments.into_iter().map(Variable::from).collect();
        
        Function {
            name: lfunc.name,
            arguments,
            body: Statement::from(lfunc.body),
            ret_type: lfunc.ret_type,
        }
    }
}

impl From<LStructDef> for StructDef {
    fn from(lstruct: LStructDef) -> Self {
        let fields = lstruct.fields.into_iter().map(Variable::from).collect();
        let methods = lstruct.methods.into_iter().map(Function::from).collect();
        
        StructDef {
            name: lstruct.name,
            fields,
            methods,
        }
    }
}

impl From<LVariable> for Variable {
    fn from(lvar: LVariable) -> Self {
        Variable {
            name: lvar.name,
            ty: lvar.ty,
        }
    }
}

impl From<LStatement> for Statement {
    fn from(lstmt: LStatement) -> Self {
        match lstmt {
            LStatement::Block { statements, scope } => {
                let statements = statements.into_iter().map(Statement::from).collect();
                let scope = scope.into_iter().map(Variable::from).collect();
                Statement::Block { statements, scope }
            }
            LStatement::Declare { variable, value } => {
                let variable = Variable::from(variable);
                let value = value.map(Expression::from);
                Statement::Declare { variable, value }
            }
            LStatement::Assign { lhs, rhs } => Statement::Assign {
                lhs: Box::new(Expression::from(*lhs)),
                rhs: Box::new(Expression::from(*rhs)),
            },
            LStatement::Return(lexpr) => Statement::Return(lexpr.map(Expression::from)),
            LStatement::If { condition, body, else_branch } => Statement::If {
                condition: Expression::from(condition),
                body: Box::new(Statement::from(*body)),
                else_branch: else_branch.map(|stmt| Box::new(Statement::from(*stmt))),
            },
            LStatement::While { condition, body } => Statement::While {
                condition: Expression::from(condition),
                body: Box::new(Statement::from(*body)),
            },
            LStatement::For { ident, expr, body } => Statement::For {
                ident: Variable::from(ident),
                expr: Expression::from(expr),
                body: Box::new(Statement::from(*body)),
            },
            LStatement::Break => Statement::Break,
            LStatement::Continue => Statement::Continue,
            LStatement::Exp(lexpr) => Statement::Exp(Expression::from(lexpr)),
        }
    }
}

impl From<LExpression> for Expression {
    fn from(lexpr: LExpression) -> Self {
        match lexpr {
            LExpression::Int(val) => Expression::Int(val),
            LExpression::Str(val) => Expression::Str(val),
            LExpression::Bool(val) => Expression::Bool(val),
            LExpression::Selff => Expression::Selff,
            LExpression::Array { capacity, elements } => {
                let elements = elements.into_iter().map(Expression::from).collect();
                Expression::Array { capacity, elements }
            }
            LExpression::FunctionCall { fn_name, args } => {
                let args = args.into_iter().map(Expression::from).collect();
                Expression::FunctionCall { fn_name, args }
            }
            LExpression::Variable(name) => Expression::Variable(name),
            LExpression::ArrayAccess { name, index } => Expression::ArrayAccess {
                name,
                index: Box::new(Expression::from(*index)),
            },
            LExpression::BinOp { lhs, op, rhs } => Expression::BinOp {
                lhs: Box::new(Expression::from(*lhs)),
                op: BinOp::from(op),
                rhs: Box::new(Expression::from(*rhs)),
            },
            LExpression::StructInitialization { name, fields } => {
                let fields = fields
                    .into_iter()
                    .map(|(k, v)| (k, Box::new(Expression::from(*v))))
                    .collect();
                Expression::StructInitialization { name, fields }
            }
            LExpression::FieldAccess { expr, field } => Expression::FieldAccess {
                expr: Box::new(Expression::from(*expr)),
                field: Box::new(Expression::from(*field)),
            },
        }
    }
}

impl From<LBinOp> for BinOp {
    fn from(lop: LBinOp) -> Self {
        match lop {
            LBinOp::Addition => BinOp::Addition,
            LBinOp::Subtraction => BinOp::Subtraction,
            LBinOp::Multiplication => BinOp::Multiplication,
            LBinOp::Division => BinOp::Division,
            LBinOp::Modulus => BinOp::Modulus,
            LBinOp::LessThan => BinOp::LessThan,
            LBinOp::LessThanOrEqual => BinOp::LessThanOrEqual,
            LBinOp::GreaterThan => BinOp::GreaterThan,
            LBinOp::GreaterThanOrEqual => BinOp::GreaterThanOrEqual,
            LBinOp::Equal => BinOp::Equal,
            LBinOp::NotEqual => BinOp::NotEqual,
            LBinOp::And => BinOp::And,
            LBinOp::Or => BinOp::Or,
            LBinOp::AddAssign => BinOp::AddAssign,
            LBinOp::SubtractAssign => BinOp::SubtractAssign,
            LBinOp::MultiplyAssign => BinOp::MultiplyAssign,
            LBinOp::DivideAssign => BinOp::DivideAssign,
        }
    }
}