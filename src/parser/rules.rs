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
use super::node_type::Statement;
use super::node_type::*;
use super::parser::Parser;
use crate::lexer::Keyword;
use crate::lexer::{TokenKind, Value};
use std::convert::TryFrom;

impl Parser {
    pub(super) fn parse_program(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();
        let globals = Vec::new();

        while self.has_more() {
            functions.push(self.parse_function()?)
        }

        Ok(Program {
            func: functions,
            globals: globals,
        })
    }

    fn parse_block(&mut self) -> Result<Statement, String> {
        self.match_token(TokenKind::CurlyBracesOpen)?;

        let mut statements = vec![];

        while let Err(_) = self.peek_token(TokenKind::CurlyBracesClose) {
            let statement = self.parse_statement()?;
            statements.push(statement);
        }

        self.match_token(TokenKind::CurlyBracesClose)?;

        Ok(Statement::Block(statements))
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        self.match_keyword(Keyword::Function)?;
        let name = self.match_identifier()?;

        self.match_token(TokenKind::BraceOpen)?;

        let arguments: Vec<Variable> = match self.peek()? {
            t if t.kind == TokenKind::BraceClose => Vec::new(),
            _ => self.parse_arguments()?,
        };

        self.match_token(TokenKind::BraceClose)?;

        let ty = match self.peek()?.kind {
            TokenKind::Colon => Some(self.parse_type()?),
            _ => None,
        };

        let body = self.parse_block()?;

        Ok(Function {
            name: name,
            arguments: arguments,
            body: body,
            ret_type: ty,
        })
    }

    fn parse_arguments(&mut self) -> Result<Vec<Variable>, String> {
        let mut args = Vec::new();
        while let Err(_) = self.peek_token(TokenKind::BraceClose) {
            let next = self.next()?;
            match next.kind {
                TokenKind::Comma => {
                    continue;
                }
                TokenKind::Identifier(name) => {
                    args.push(Variable {
                        name: name,
                        ty: Some(self.parse_type()?),
                    });
                }
                _ => return Err(self.make_error(TokenKind::Identifier("Argument".into()), next)),
            }
        }

        Ok(args)
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        self.match_token(TokenKind::Colon)?;
        let next = self.peek()?;
        let typ = match next.kind {
            TokenKind::Identifier(_) => Type::try_from(self.next()?.raw),
            _ => Err("Expected type".into()),
        }?;
        if let Ok(_) = self.peek_token(TokenKind::SquareBraceOpen) {
            self.match_token(TokenKind::SquareBraceOpen)?;
            self.match_token(TokenKind::SquareBraceClose)?;
            Ok(Type::Array(Box::new(typ)))
        } else {
            Ok(typ)
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let token = self.peek()?;
        let state = match &token.kind {
            TokenKind::Keyword(Keyword::Let) => self.parse_declare(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return(),
            TokenKind::Keyword(Keyword::If) => self.parse_conditional_statement(),
            TokenKind::Keyword(Keyword::While) => self.parse_while_loop(),
            TokenKind::Keyword(Keyword::For) => self.parse_for_loop(),
            TokenKind::Identifier(_) => {
                let ident = self.match_identifier()?;

                if let Ok(_) = self.peek_token(TokenKind::BraceOpen) {
                    let state = self.parse_function_call(Some(ident))?;
                    Ok(Statement::Exp(state))
                } else if let Ok(_) = self.peek_token(TokenKind::Assign) {
                    let state = self.parse_assignent(Some(Expression::Variable(ident)))?;
                    Ok(state)
                } else if let Ok(_) = self.peek_token(TokenKind::SquareBraceOpen) {
                    let expr = self.parse_array_access(Some(ident))?;

                    let next = self.peek()?;
                    match next.kind {
                        TokenKind::Assign => self.parse_assignent(Some(expr)),
                        _ => Ok(Statement::Exp(expr)),
                    }
                } else {
                    let state = Statement::Exp(Expression::Variable(ident.into()));
                    Ok(state)
                }
            }
            TokenKind::Literal(_) => Ok(Statement::Exp(self.parse_expression()?)),
            _ => return Err(self.make_error(TokenKind::Unknown, token)),
        };
        state
    }

    /// Parses a function call from tokens.
    /// The name of the function needs to be passed here, because we have already passed it with our cursor.
    /// If no function name is provided, the next token will be fetched
    fn parse_function_call(&mut self, func_name: Option<String>) -> Result<Expression, String> {
        let name = match func_name {
            Some(name) => name,
            None => self.next()?.raw,
        };

        self.match_token(TokenKind::BraceOpen)?;

        let mut args = Vec::new();

        loop {
            let next = self.peek()?;
            match &next.kind {
                TokenKind::BraceClose => break,
                TokenKind::Comma => {
                    let _ = self.next();
                    continue;
                }
                TokenKind::Identifier(_) | TokenKind::Literal(_) => {
                    args.push(self.parse_expression()?)
                }
                TokenKind::Keyword(Keyword::Boolean) => args.push(self.parse_expression()?),
                _ => {
                    return Err(self.make_error(TokenKind::BraceClose, next));
                }
            };
        }

        self.match_token(TokenKind::BraceClose)?;
        Ok(Expression::FunctionCall(name, args))
    }

    fn parse_return(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::Return)?;
        let peeked = self.peek()?;
        match peeked.kind {
            TokenKind::SemiColon => Ok(Statement::Return(None)),
            _ => Ok(Statement::Return(Some(self.parse_expression()?))),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        let token = self.next()?;
        match token.kind {
            TokenKind::BraceOpen => {
                let expr = self.parse_expression()?;
                self.match_token(TokenKind::BraceClose)?;
                Ok(expr)
            }
            TokenKind::Keyword(Keyword::Boolean) => {
                let state = match BinOp::try_from(self.peek()?.kind) {
                    Ok(_) => self.parse_bin_op(None)?,
                    Err(_) => {
                        Expression::Bool(token.raw.parse::<bool>().map_err(|e| e.to_string())?)
                    }
                };
                Ok(state)
            }
            TokenKind::Literal(Value::Int) => {
                let state = match BinOp::try_from(self.peek()?.kind) {
                    Ok(_) => self.parse_bin_op(None)?,
                    Err(_) => Expression::Int(token.raw.parse::<u32>().map_err(|e| e.to_string())?),
                };
                Ok(state)
            }
            TokenKind::Literal(Value::Str) => {
                let state = match BinOp::try_from(self.peek()?.kind) {
                    Ok(_) => self.parse_bin_op(None)?,
                    Err(_) => Expression::Str(token.raw),
                };
                Ok(state)
            }
            TokenKind::Identifier(val) => {
                let next = self.peek()?;
                let state = match &next.kind {
                    TokenKind::BraceOpen => {
                        let func_call = self.parse_function_call(Some(val))?;
                        match BinOp::try_from(self.peek()?.kind) {
                            Ok(_) => self.parse_bin_op(Some(func_call))?,
                            Err(_) => func_call,
                        }
                    }
                    TokenKind::SquareBraceOpen => {
                        let arr = self.parse_array_access(Some(val))?;
                        match BinOp::try_from(self.peek()?.kind) {
                            Ok(_) => self.parse_bin_op(Some(arr))?,
                            Err(_) => arr,
                        }
                    }
                    _ => match BinOp::try_from(self.peek()?.kind) {
                        Ok(_) => self.parse_bin_op(Some(Expression::Variable(token.raw)))?,
                        Err(_) => Expression::Variable(val),
                    },
                };
                Ok(state)
            }
            TokenKind::SquareBraceOpen => self.parse_array(),
            other => Err(format!("Expected Expression, found {:?}", other)),
        }
    }

    fn parse_array(&mut self) -> Result<Expression, String> {
        let mut elements = Vec::new();
        loop {
            let next = self.peek()?;
            match next.kind {
                TokenKind::SquareBraceClose => {}
                TokenKind::Literal(Value::Int) => {
                    let value = self.next()?.raw.parse::<u32>().map_err(|e| e.to_string())?;
                    elements.push(Expression::Int(value));
                }
                TokenKind::Literal(Value::Str) => {
                    elements.push(Expression::Str(self.next()?.raw));
                }
                _ => {
                    let n = self.next()?;
                    return Err(self.make_error(TokenKind::Identifier("Argument".into()), n));
                }
            };
            if self.peek_token(TokenKind::SquareBraceClose).is_ok() {
                break;
            }
            self.match_token(TokenKind::Comma)?;
        }

        self.match_token(TokenKind::SquareBraceClose)?;

        Ok(Expression::Array(elements))
    }

    fn parse_array_access(&mut self, arr_name: Option<String>) -> Result<Expression, String> {
        let name = match arr_name {
            Some(name) => name,
            None => self.next()?.raw,
        };

        self.match_token(TokenKind::SquareBraceOpen)?;
        let expr = self.parse_expression()?;
        self.match_token(TokenKind::SquareBraceClose)?;

        Ok(Expression::ArrayAccess(name, Box::new(expr)))
    }

    fn parse_while_loop(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::While)?;
        let expr = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::While(expr, Box::new(body)))
    }

    fn parse_for_loop(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::For)?;

        let ident = self.match_identifier()?;
        self.match_keyword(Keyword::In)?;
        let expr = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::For(
            Variable {
                name: ident,
                ty: None,
            },
            expr,
            Box::new(body),
        ))
    }

    fn parse_conditional_statement(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::If)?;
        let condition = self.parse_expression()?;

        let body = self.parse_block()?;

        match self.peek()? {
            tok if tok.kind == TokenKind::Keyword(Keyword::Else) => {
                let _ = self.next();

                let peeked = self.peek()?;

                let has_else = match &peeked.kind {
                    TokenKind::CurlyBracesOpen => Some(self.parse_block()?),
                    _ => None,
                };

                let else_branch = match has_else {
                    Some(branch) => branch,
                    None => self.parse_conditional_statement()?,
                };
                Ok(Statement::If(
                    condition,
                    Box::new(body),
                    Some(Box::new(else_branch)),
                ))
            }
            _ => Ok(Statement::If(condition, Box::new(body), None)),
        }
    }

    /// In some occurences a complex expression has been evaluated before a binary operation is encountered.
    /// The following expression is one such example:
    /// ```
    /// foo(1) * 2
    /// ```
    /// In this case, the function call has already been evaluated, and needs to be passed to this function.
    fn parse_bin_op(&mut self, lhs: Option<Expression>) -> Result<Expression, String> {
        let left = match lhs {
            Some(lhs) => lhs,
            None => {
                let prev = self.prev().ok_or_else(|| "Expected Token")?;
                match &prev.kind {
                    TokenKind::Identifier(_) | TokenKind::Literal(_) | TokenKind::Keyword(_) => {
                        Ok(Expression::try_from(prev)?)
                    }
                    _ => Err(self.make_error(TokenKind::Unknown, prev)),
                }?
            }
        };

        let op = self.match_operator()?;

        Ok(Expression::BinOp(
            Box::from(Expression::try_from(left).map_err(|e| e.to_string())?),
            op,
            Box::from(self.parse_expression()?),
        ))
    }

    fn parse_declare(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::Let)?;
        let name = self.match_identifier()?;
        let ty = match self.peek()?.kind {
            TokenKind::Colon => Some(self.parse_type()?),
            _ => None,
        };

        match self.peek()?.kind {
            TokenKind::Assign => {
                self.match_token(TokenKind::Assign)?;
                let expr = self.parse_expression()?;
                Ok(Statement::Declare(Variable { name, ty }, Some(expr)))
            }
            _ => Ok(Statement::Declare(Variable { name, ty }, None)),
        }
    }

    fn parse_assignent(&mut self, name: Option<Expression>) -> Result<Statement, String> {
        let name = match name {
            Some(name) => name,
            None => Expression::Variable(self.match_identifier()?),
        };

        self.match_token(TokenKind::Assign)?;

        let expr = self.parse_expression()?;

        Ok(Statement::Assign(Box::new(name), Box::new(expr)))
    }
}
