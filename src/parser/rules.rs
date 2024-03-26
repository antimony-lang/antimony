use super::parser::Parser;
use crate::ast::types::Type;
use crate::ast::*;
use crate::lexer::Keyword;
use crate::lexer::{TokenKind, Value};
use std::collections::HashMap;
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
use std::collections::HashSet;
use std::convert::TryFrom;

impl Parser {
    pub fn parse_module(&mut self) -> Result<Module, String> {
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut imports = HashSet::new();
        let globals = Vec::new();

        while self.has_more() {
            let next = self.peek()?;
            match next.kind {
                TokenKind::Keyword(Keyword::Function) => functions.push(self.parse_function()?),
                TokenKind::Keyword(Keyword::Import) => {
                    imports.insert(self.parse_import()?);
                }
                TokenKind::Keyword(Keyword::Struct) => {
                    structs.push(self.parse_struct_definition()?)
                }
                _ => return Err(format!("Unexpected token: {}", next.raw)),
            }
        }

        // TODO: Populate imports

        Ok(Module {
            func: functions,
            structs,
            globals,
            path: self.path.clone(),
            imports,
        })
    }

    fn parse_struct_definition(&mut self) -> Result<StructDef, String> {
        self.match_keyword(Keyword::Struct)?;
        let name = self.match_identifier()?;

        self.match_token(TokenKind::CurlyBracesOpen)?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        while self.peek_token(TokenKind::CurlyBracesClose).is_err() {
            let next = self.peek()?;
            match next.kind {
                TokenKind::Keyword(Keyword::Function) => {
                    methods.push(self.parse_function()?);
                }
                TokenKind::Identifier(_) => fields.push(self.parse_typed_variable()?),
                _ => {
                    return Err(
                        self.make_error_msg(next.pos, "Expected struct field or method".into())
                    )
                }
            }
        }
        self.match_token(TokenKind::CurlyBracesClose)?;
        Ok(StructDef {
            name,
            fields,
            methods,
        })
    }

    fn parse_typed_variable_list(&mut self) -> Result<Vec<TypedVariable>, String> {
        let mut args = Vec::new();

        // If there is an argument
        if let TokenKind::Identifier(_) = self.peek()?.kind {
            // Parse first argument
            args.push(self.parse_typed_variable()?);
            // Then continue to parse arguments
            // as long as a comma token is found
            while self.peek_token(TokenKind::Comma).is_ok() {
                self.match_token(TokenKind::Comma)?;
                args.push(self.parse_typed_variable()?);
            }
        }

        Ok(args)
    }

    fn parse_typed_variable(&mut self) -> Result<TypedVariable, String> {
        let next = self.next()?;
        if let TokenKind::Identifier(name) = next.kind {
            return Ok(TypedVariable {
                name,
                ty: self.parse_type()?,
            });
        }

        Err(format!("Argument could not be parsed: {}", next.raw))
    }

    fn parse_block(&mut self) -> Result<Statement, String> {
        self.match_token(TokenKind::CurlyBracesOpen)?;

        let mut statements = vec![];
        let mut scope = vec![];

        // Parse statements until a curly brace is encountered
        while self.peek_token(TokenKind::CurlyBracesClose).is_err() {
            let statement = self.parse_statement()?;

            // If the current statement is a variable declaration,
            // let the scope know
            if let Statement::Declare { variable, value: _ } = &statement {
                // TODO: Not sure if we should clone here
                scope.push(variable.to_owned());
            }

            statements.push(statement);
        }

        self.match_token(TokenKind::CurlyBracesClose)?;

        Ok(Statement::Block { statements, scope })
    }

    /// To reduce code duplication, this method can be either be used to parse a function or a method.
    /// If a function is parsed, the `fn` keyword is matched.
    /// If a method is parsed, `fn` will be omitted
    fn parse_function(&mut self) -> Result<Function, String> {
        self.match_keyword(Keyword::Function)?;
        let name = self.match_identifier()?;

        self.match_token(TokenKind::BraceOpen)?;

        let arguments: Vec<TypedVariable> = match self.peek()? {
            t if t.kind == TokenKind::BraceClose => Vec::new(),
            _ => self.parse_typed_variable_list()?,
        };

        self.match_token(TokenKind::BraceClose)?;

        let ty = match self.peek()?.kind {
            TokenKind::Colon => Some(self.parse_type()?),
            _ => None,
        };

        let body = self.parse_block()?;

        Ok(Function {
            name,
            arguments,
            body,
            ret_type: ty,
        })
    }

    fn parse_import(&mut self) -> Result<String, String> {
        self.match_keyword(Keyword::Import)?;
        let token = self.next()?;
        let path = match token.kind {
            TokenKind::Literal(Value::Str(path)) => path,
            other => {
                return Err(
                    self.make_error_msg(token.pos, format!("Expected string, got {:?}", other))
                )
            }
        };

        Ok(path)
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        self.match_token(TokenKind::Colon)?;
        let next = self.peek()?;
        let typ = match next.kind {
            TokenKind::Identifier(_) => Type::try_from(self.next()?.raw),
            _ => Err("Expected type".into()),
        }?;
        if self.peek_token(TokenKind::SquareBraceOpen).is_ok() {
            self.match_token(TokenKind::SquareBraceOpen)?;
            let capacity = match self.peek_token(TokenKind::Literal(Value::Int)) {
                Ok(val) => {
                    self.next()?;
                    val.raw.parse().ok()
                }
                Err(_) => None,
            };
            self.match_token(TokenKind::SquareBraceClose)?;
            Ok(Type::Array(Box::new(typ), capacity))
        } else {
            Ok(typ)
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let token = self.peek()?;
        let expr = match &token.kind {
            TokenKind::CurlyBracesOpen => return self.parse_block(),
            TokenKind::Keyword(Keyword::Let) => return self.parse_declare(),
            TokenKind::Keyword(Keyword::Return) => return self.parse_return(),
            TokenKind::Keyword(Keyword::If) => return self.parse_conditional_statement(),
            TokenKind::Keyword(Keyword::While) => return self.parse_while_loop(),
            TokenKind::Keyword(Keyword::Break) => return self.parse_break(),
            TokenKind::Keyword(Keyword::Continue) => return self.parse_continue(),
            TokenKind::Keyword(Keyword::For) => return self.parse_for_loop(),
            TokenKind::Keyword(Keyword::Match) => return self.parse_match_statement(),
            TokenKind::BraceOpen
            | TokenKind::Keyword(Keyword::Selff)
            | TokenKind::Identifier(_)
            | TokenKind::Literal(_) => self.parse_expression()?,
            TokenKind::Keyword(Keyword::Struct) => {
                return Err("Struct definitions inside functions are not allowed".to_string())
            }
            _ => {
                return Err(self.make_error_msg(token.pos, "Failed to parse statement".to_string()))
            }
        };
        let suffix = self.peek()?;
        if AssignOp::try_from(suffix.kind).is_ok() {
            Ok(self.parse_assignment(expr)?)
        } else {
            Ok(Statement::Exp(expr))
        }
    }

    fn parse_function_call(&mut self, expr: Expression) -> Result<Expression, String> {
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
                TokenKind::Keyword(Keyword::Boolean) | TokenKind::Keyword(Keyword::New) => {
                    args.push(self.parse_expression()?)
                }
                TokenKind::SquareBraceOpen => {
                    // TODO: Expression parsing currently uses `next` instead of `peek`.
                    // We have to eat that token here until that is resolved
                    self.match_token(TokenKind::SquareBraceOpen)?;
                    args.push(self.parse_array()?);
                }
                _ => {
                    return Err(self.make_error(TokenKind::BraceClose, next));
                }
            };
        }

        self.match_token(TokenKind::BraceClose)?;
        Ok(Expression::FunctionCall {
            expr: Box::new(expr),
            args,
        })
    }

    fn parse_return(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::Return)?;
        let peeked = self.peek()?;
        match peeked.kind {
            TokenKind::SemiColon => {
                self.next()?;
                Ok(Statement::Return(None))
            }
            _ => Ok(Statement::Return(Some(self.parse_expression()?))),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        let token = self.next()?;

        // TODO: don't mut
        let mut expr = match token.kind {
            // (1 + 2)
            TokenKind::BraceOpen => {
                let expr = self.parse_expression()?;
                self.match_token(TokenKind::BraceClose)?;
                expr
            }
            // true | false
            TokenKind::Keyword(Keyword::Boolean) => {
                Expression::Bool(token.raw.parse::<bool>().map_err(|e| e.to_string())?)
            }
            // 5
            TokenKind::Literal(Value::Int) => {
                // Ignore spacing character (E.g. 1_000_000)
                let clean_str = token.raw.replace('_', "");
                let val = match clean_str {
                    c if c.starts_with("0b") => {
                        usize::from_str_radix(token.raw.trim_start_matches("0b"), 2)
                            .map_err(|e| e.to_string())?
                    }
                    c if c.starts_with("0o") => {
                        usize::from_str_radix(token.raw.trim_start_matches("0o"), 8)
                            .map_err(|e| e.to_string())?
                    }
                    c if c.starts_with("0x") => {
                        usize::from_str_radix(token.raw.trim_start_matches("0x"), 16)
                            .map_err(|e| e.to_string())?
                    }
                    c => c.parse::<usize>().map_err(|e| e.to_string())?,
                };
                Expression::Int(val)
            }
            // "A string"
            TokenKind::Literal(Value::Str(string)) => Expression::Str(string),
            // self
            TokenKind::Keyword(Keyword::Selff) => Expression::Selff,
            // name
            TokenKind::Identifier(val) => Expression::Variable(val),
            // [1, 2, 3]
            TokenKind::SquareBraceOpen => self.parse_array()?,
            // new Foo {}
            TokenKind::Keyword(Keyword::New) => self.parse_struct_initialization()?,
            other => return Err(format!("Expected Expression, found {:?}", other)),
        };

        // Check if the parsed expression continues
        loop {
            if self.peek_token(TokenKind::Dot).is_ok() {
                // foo.bar
                expr = self.parse_field_access(expr)?;
            } else if self.peek_token(TokenKind::SquareBraceOpen).is_ok() {
                // foo[0]
                expr = self.parse_array_access(expr)?;
            } else if self.peek_token(TokenKind::BraceOpen).is_ok() {
                // foo(a, b)
                expr = self.parse_function_call(expr)?;
            } else if BinOp::try_from(self.peek()?.kind).is_ok() {
                // a + b
                expr = self.parse_bin_op(expr)?;
            } else {
                // The expression was fully parsed
                return Ok(expr);
            }
        }
    }

    fn parse_field_access(&mut self, lhs: Expression) -> Result<Expression, String> {
        self.match_token(TokenKind::Dot)?;

        let field = self.match_identifier()?;
        let expr = Expression::FieldAccess {
            expr: Box::new(lhs),
            field,
        };
        Ok(expr)
    }

    /// TODO: Cleanup
    fn parse_struct_initialization(&mut self) -> Result<Expression, String> {
        let name = self.match_identifier()?;
        self.match_token(TokenKind::CurlyBracesOpen)?;
        let fields = self.parse_struct_fields()?;
        self.match_token(TokenKind::CurlyBracesClose)?;

        Ok(Expression::StructInitialization { name, fields })
    }

    fn parse_struct_fields(&mut self) -> Result<HashMap<String, Box<Expression>>, String> {
        let mut map = HashMap::new();

        // If there is a field
        if let TokenKind::Identifier(_) = self.peek()?.kind {
            // Parse first field
            let (name, expr) = self.parse_struct_field()?;
            map.insert(name, expr);
            // Then continue to parse fields
            // as long as a comma token is found
            while matches!(self.peek()?.kind, TokenKind::Identifier(_)) {
                let (name, expr) = self.parse_struct_field()?;
                map.insert(name, expr);
            }
        }

        Ok(map)
    }

    fn parse_struct_field(&mut self) -> Result<(String, Box<Expression>), String> {
        let next = self.next()?;
        if let TokenKind::Identifier(name) = next.kind {
            self.match_token(TokenKind::Colon)?;
            return Ok((name, Box::new(self.parse_expression()?)));
        }

        Err(format!("Struct field could not be parsed: {}", next.raw))
    }

    fn parse_array(&mut self) -> Result<Expression, String> {
        let mut elements = Vec::new();
        loop {
            let next = self.peek()?;
            match next.kind {
                TokenKind::SquareBraceClose => {}
                TokenKind::Literal(Value::Int) => {
                    let value = self
                        .next()?
                        .raw
                        .parse::<usize>()
                        .map_err(|e| e.to_string())?;
                    elements.push(Expression::Int(value));
                }
                _ => {
                    let expr = self.parse_expression()?;
                    elements.push(expr);
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

    fn parse_array_access(&mut self, expr: Expression) -> Result<Expression, String> {
        self.match_token(TokenKind::SquareBraceOpen)?;
        let index = self.parse_expression()?;
        self.match_token(TokenKind::SquareBraceClose)?;

        Ok(Expression::ArrayAccess {
            expr: Box::new(expr),
            index: Box::new(index),
        })
    }

    fn parse_while_loop(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::While)?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::While {
            condition,
            body: Box::new(body),
        })
    }

    fn parse_break(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::Break)?;
        Ok(Statement::Break)
    }

    fn parse_continue(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::Continue)?;
        Ok(Statement::Continue)
    }

    fn parse_for_loop(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::For)?;

        let ident = self.match_identifier()?;
        let ident_ty = match self.peek()?.kind {
            TokenKind::Colon => Some(self.parse_type()?),
            _ => None,
        };
        self.match_keyword(Keyword::In)?;
        let expr = self.parse_expression()?;

        let body = self.parse_block()?;

        Ok(Statement::For {
            ident: Variable {
                name: ident,
                ty: ident_ty,
            },
            expr,
            body: Box::new(body),
        })
    }

    fn parse_match_statement(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::Match)?;
        let subject = self.parse_expression()?;
        self.match_token(TokenKind::CurlyBracesOpen)?;
        let mut arms: Vec<MatchArm> = Vec::new();

        // Used to mitigate multiple else cases were defined
        let mut has_else = false;
        loop {
            let next = self.peek()?;
            match next.kind {
                TokenKind::Literal(_)
                | TokenKind::Identifier(_)
                | TokenKind::Keyword(Keyword::Boolean) => arms.push(self.parse_match_arm()?),
                TokenKind::Keyword(Keyword::Else) => {
                    if has_else {
                        return Err(self.make_error_msg(
                            next.pos,
                            "Multiple else arms are not allowed".to_string(),
                        ));
                    }
                    has_else = true;
                    arms.push(self.parse_match_arm()?);
                }
                TokenKind::CurlyBracesClose => break,
                _ => return Err(self.make_error_msg(next.pos, "Illegal token".to_string())),
            }
        }
        self.match_token(TokenKind::CurlyBracesClose)?;
        Ok(Statement::Match { subject, arms })
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, String> {
        let next = self.peek()?;

        match next.kind {
            TokenKind::Keyword(Keyword::Else) => {
                self.match_keyword(Keyword::Else)?;
                self.match_token(TokenKind::ArrowRight)?;
                Ok(MatchArm::Else(self.parse_statement()?))
            }
            _ => {
                let expr = self.parse_expression()?;
                self.match_token(TokenKind::ArrowRight)?;
                let statement = self.parse_statement()?;

                Ok(MatchArm::Case(expr, statement))
            }
        }
    }

    fn parse_conditional_statement(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::If)?;
        let condition = self.parse_expression()?;

        let body = self.parse_block()?;

        match self.peek()? {
            tok if tok.kind == TokenKind::Keyword(Keyword::Else) => {
                let _ = self.next();

                let peeked = self.peek()?;

                let else_branch = match &peeked.kind {
                    TokenKind::CurlyBracesOpen => Some(self.parse_block()?),
                    _ => None,
                };

                let else_branch = match else_branch {
                    Some(branch) => branch,
                    None => self.parse_conditional_statement()?,
                };
                Ok(Statement::If {
                    condition,
                    body: Box::new(body),
                    else_branch: Some(Box::new(else_branch)),
                })
            }
            _ => Ok(Statement::If {
                condition,
                body: Box::new(body),
                else_branch: None,
            }),
        }
    }

    /// In some occurences a complex expression has been evaluated before a binary operation is encountered.
    /// The following expression is one such example:
    /// ```
    /// foo(1) * 2
    /// ```
    /// In this case, the function call has already been evaluated, and needs to be passed to this function.
    fn parse_bin_op(&mut self, lhs: Expression) -> Result<Expression, String> {
        let op = self.match_operator()?;

        Ok(Expression::BinOp {
            lhs: Box::from(lhs),
            op,
            rhs: Box::from(self.parse_expression()?),
        })
    }

    fn parse_declare(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::Let)?;
        let name = self.match_identifier()?;
        let token = self.peek()?;
        let ty = match &token.kind {
            TokenKind::Colon => Some(self.parse_type()?),
            TokenKind::Assign => None,
            _ => {
                // FIXME: context for this error message is not ideal
                return Err(self.make_error_msg(
                    token.pos,
                    format!("Expected ':' or '=', found {:?}", token.kind),
                ));
            }
        };

        match self.peek()?.kind {
            TokenKind::Assign => {
                self.match_token(TokenKind::Assign)?;
                let expr = self.parse_expression()?;
                Ok(Statement::Declare {
                    variable: Variable { name, ty },
                    value: Some(expr),
                })
            }
            _ => Ok(Statement::Declare {
                variable: Variable { name, ty },
                value: None,
            }),
        }
    }

    fn parse_assignment(&mut self, lhs: Expression) -> Result<Statement, String> {
        let op = AssignOp::try_from(self.next()?.kind).unwrap();

        let expr = self.parse_expression()?;

        Ok(Statement::Assign {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(expr),
        })
    }
}
