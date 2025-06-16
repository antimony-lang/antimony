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

use super::hast::*;
use super::llast::*;
use std::collections::HashMap;

/// Transforms high-level AST to low-level AST
/// This involves lowering complex constructs like match statements
/// to simpler constructs that backends can easily handle
pub struct AstTransformer;

impl AstTransformer {
    pub fn transform_module(hmodule: HModule) -> Result<Module, String> {
        let mut func = Vec::new();
        let mut structs = Vec::new();

        for hfunc in hmodule.func {
            func.push(Self::transform_function(hfunc)?);
        }

        for hstruct in hmodule.structs {
            structs.push(Self::transform_struct_def(hstruct)?);
        }

        Ok(Module {
            func,
            structs,
            globals: hmodule.globals,
        })
    }

    fn transform_function(hfunc: HFunction) -> Result<Function, String> {
        let mut arguments = Vec::new();
        for harg in hfunc.arguments {
            arguments.push(Self::transform_variable(harg));
        }

        Ok(Function {
            name: hfunc.name,
            arguments,
            body: Self::transform_statement(hfunc.body)?,
            ret_type: hfunc.ret_type,
        })
    }

    fn transform_struct_def(hstruct: HStructDef) -> Result<StructDef, String> {
        let mut fields = Vec::new();
        let mut methods = Vec::new();

        for hfield in hstruct.fields {
            fields.push(Self::transform_variable(hfield));
        }

        for hmethod in hstruct.methods {
            methods.push(Self::transform_function(hmethod)?);
        }

        Ok(StructDef {
            name: hstruct.name,
            fields,
            methods,
        })
    }

    fn transform_variable(hvar: HVariable) -> Variable {
        Variable {
            name: hvar.name,
            ty: hvar.ty,
        }
    }

    fn transform_statement(hstmt: HStatement) -> Result<Statement, String> {
        match hstmt {
            HStatement::Block { statements, scope } => {
                let mut lstmts = Vec::new();
                let mut lscope = Vec::new();

                for hstmt in statements {
                    lstmts.push(Self::transform_statement(hstmt)?);
                }

                for hvar in scope {
                    lscope.push(Self::transform_variable(hvar));
                }

                Ok(Statement::Block {
                    statements: lstmts,
                    scope: lscope,
                })
            }
            HStatement::Declare { variable, value } => {
                let lvar = Self::transform_variable(variable);
                let lvalue = match value {
                    Some(hexpr) => Some(Self::transform_expression(hexpr)?),
                    None => None,
                };

                Ok(Statement::Declare {
                    variable: lvar,
                    value: lvalue,
                })
            }
            HStatement::Assign { lhs, rhs } => Ok(Statement::Assign {
                lhs: Box::new(Self::transform_expression(*lhs)?),
                rhs: Box::new(Self::transform_expression(*rhs)?),
            }),
            HStatement::Return(hexpr) => {
                let lexpr = match hexpr {
                    Some(expr) => Some(Self::transform_expression(expr)?),
                    None => None,
                };
                Ok(Statement::Return(lexpr))
            }
            HStatement::If {
                condition,
                body,
                else_branch,
            } => {
                let lcond = Self::transform_expression(condition)?;
                let lbody = Box::new(Self::transform_statement(*body)?);
                let lelse = match else_branch {
                    Some(else_stmt) => Some(Box::new(Self::transform_statement(*else_stmt)?)),
                    None => None,
                };

                Ok(Statement::If {
                    condition: lcond,
                    body: lbody,
                    else_branch: lelse,
                })
            }
            HStatement::While { condition, body } => Ok(Statement::While {
                condition: Self::transform_expression(condition)?,
                body: Box::new(Self::transform_statement(*body)?),
            }),
            HStatement::For { ident, expr, body } => Ok(Statement::For {
                ident: Self::transform_variable(ident),
                expr: Self::transform_expression(expr)?,
                body: Box::new(Self::transform_statement(*body)?),
            }),
            // This is the key transformation: match -> if-else chain
            HStatement::Match { subject, arms } => {
                Self::transform_match_to_if_else(subject, arms)
            }
            HStatement::Break => Ok(Statement::Break),
            HStatement::Continue => Ok(Statement::Continue),
            HStatement::Exp(hexpr) => Ok(Statement::Exp(Self::transform_expression(hexpr)?)),
        }
    }

    /// Transforms a match statement into a chain of if-else statements
    /// This is the core lowering that enables high-level match syntax
    /// while keeping backends simple
    fn transform_match_to_if_else(
        subject: HExpression,
        arms: Vec<HMatchArm>,
    ) -> Result<Statement, String> {
        if arms.is_empty() {
            return Err("Match statement must have at least one arm".to_string());
        }

        let lsubject = Self::transform_expression(subject)?;

        // Build if-else chain from match arms
        let mut current_stmt: Option<Statement> = None;

        // Process arms in reverse order to build nested if-else structure
        for arm in arms.into_iter().rev() {
            match arm {
                HMatchArm::Case(pattern, body) => {
                    let lpattern = Self::transform_expression(pattern)?;
                    let lbody = Self::transform_statement(body)?;

                    // Create equality check: subject == pattern
                    let condition = Expression::BinOp {
                        lhs: Box::new(lsubject.clone()),
                        op: BinOp::Equal,
                        rhs: Box::new(lpattern),
                    };

                    // Wrap single statements in blocks for generator compatibility
                    let body_block = match lbody {
                        Statement::Block { .. } => lbody,
                        other => Statement::Block {
                            statements: vec![other],
                            scope: vec![],
                        },
                    };

                    current_stmt = Some(Statement::If {
                        condition,
                        body: Box::new(body_block),
                        else_branch: current_stmt.map(Box::new),
                    });
                }
                HMatchArm::Else(body) => {
                    let lbody = Self::transform_statement(body)?;
                    // Wrap single statements in blocks for generator compatibility
                    let body_block = match lbody {
                        Statement::Block { .. } => lbody,
                        other => Statement::Block {
                            statements: vec![other],
                            scope: vec![],
                        },
                    };
                    current_stmt = Some(body_block);
                }
            }
        }

        current_stmt.ok_or_else(|| "Failed to transform match statement".to_string())
    }

    fn transform_expression(hexpr: HExpression) -> Result<Expression, String> {
        match hexpr {
            HExpression::Int(val) => Ok(Expression::Int(val)),
            HExpression::Str(val) => Ok(Expression::Str(val)),
            HExpression::Bool(val) => Ok(Expression::Bool(val)),
            HExpression::Selff => Ok(Expression::Selff),
            HExpression::Array { capacity, elements } => {
                let mut lelements = Vec::new();
                for helement in elements {
                    lelements.push(Self::transform_expression(helement)?);
                }
                Ok(Expression::Array {
                    capacity,
                    elements: lelements,
                })
            }
            HExpression::FunctionCall { fn_name, args } => {
                let mut largs = Vec::new();
                for harg in args {
                    largs.push(Self::transform_expression(harg)?);
                }
                Ok(Expression::FunctionCall {
                    fn_name,
                    args: largs,
                })
            }
            HExpression::Variable(name) => Ok(Expression::Variable(name)),
            HExpression::ArrayAccess { name, index } => Ok(Expression::ArrayAccess {
                name,
                index: Box::new(Self::transform_expression(*index)?),
            }),
            HExpression::BinOp { lhs, op, rhs } => Ok(Expression::BinOp {
                lhs: Box::new(Self::transform_expression(*lhs)?),
                op: Self::transform_bin_op(op),
                rhs: Box::new(Self::transform_expression(*rhs)?),
            }),
            HExpression::StructInitialization { name, fields } => {
                let mut lfields = HashMap::new();
                for (field_name, field_expr) in fields {
                    lfields.insert(
                        field_name,
                        Box::new(Self::transform_expression(*field_expr)?),
                    );
                }
                Ok(Expression::StructInitialization {
                    name,
                    fields: lfields,
                })
            }
            HExpression::FieldAccess { expr, field } => Ok(Expression::FieldAccess {
                expr: Box::new(Self::transform_expression(*expr)?),
                field: Box::new(Self::transform_expression(*field)?),
            }),
        }
    }

    fn transform_bin_op(hop: HBinOp) -> BinOp {
        match hop {
            HBinOp::Addition => BinOp::Addition,
            HBinOp::Subtraction => BinOp::Subtraction,
            HBinOp::Multiplication => BinOp::Multiplication,
            HBinOp::Division => BinOp::Division,
            HBinOp::Modulus => BinOp::Modulus,
            HBinOp::LessThan => BinOp::LessThan,
            HBinOp::LessThanOrEqual => BinOp::LessThanOrEqual,
            HBinOp::GreaterThan => BinOp::GreaterThan,
            HBinOp::GreaterThanOrEqual => BinOp::GreaterThanOrEqual,
            HBinOp::Equal => BinOp::Equal,
            HBinOp::NotEqual => BinOp::NotEqual,
            HBinOp::And => BinOp::And,
            HBinOp::Or => BinOp::Or,
            HBinOp::AddAssign => BinOp::AddAssign,
            HBinOp::SubtractAssign => BinOp::SubtractAssign,
            HBinOp::MultiplyAssign => BinOp::MultiplyAssign,
            HBinOp::DivideAssign => BinOp::DivideAssign,
        }
    }
}