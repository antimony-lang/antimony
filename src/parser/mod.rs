use crate::lexer::Keyword;
use crate::lexer::{Token, TokenKind, Value};
use crate::parser::node_type::Statement;
use crate::parser::node_type::*;
use crate::util::string_util::highlight_position_in_file;
use std::convert::TryFrom;
use std::iter::Peekable;
use std::vec::IntoIter;

pub mod node_type;

#[cfg(test)]
mod tests;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    peeked: Vec<Token>,
    current: Option<Token>,
    prev: Option<Token>,
    raw: Option<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, raw: Option<String>) -> Parser {
        // FIXME: Fiter without collecting?
        let tokens_without_whitespace: Vec<Token> = tokens
            .into_iter()
            .filter(|token| token.kind != TokenKind::Whitespace && token.kind != TokenKind::Comment)
            .collect();
        Parser {
            tokens: tokens_without_whitespace.into_iter().peekable(),
            peeked: vec![],
            current: None,
            prev: None,
            raw: raw,
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        self.parse_program()
    }

    fn next(&mut self) -> Option<Token> {
        self.prev = self.current.to_owned();
        let item = if self.peeked.is_empty() {
            self.tokens.next()
        } else {
            self.peeked.pop()
        };

        self.current = item.to_owned();
        item
    }

    fn peek(&mut self) -> Option<Token> {
        if let Some(token) = self.next() {
            self.push(Some(token.to_owned()));
            Some(token)
        } else {
            None
        }
    }

    fn drop(&mut self, count: usize) {
        for _ in 0..count {
            self.next();
        }
    }

    fn push(&mut self, token: Option<Token>) {
        if let Some(t) = token {
            self.peeked.push(t);
        }
    }

    fn has_more(&mut self) -> bool {
        !self.peeked.is_empty() || self.tokens.peek().is_some()
    }

    fn next_token(&mut self) -> Token {
        self.next().expect("failed to parse")
    }

    fn match_token(&mut self, token_kind: TokenKind) -> Result<Token, String> {
        match self.next() {
            Some(token) if token.kind == token_kind => Ok(token),
            Some(other) => Err(self.make_error(token_kind, other)),
            None => Err("Token expected".to_string()),
        }
    }

    fn peek_token(&mut self, token_kind: TokenKind) -> Result<Token, String> {
        match self.peek() {
            Some(token) if token.kind == token_kind => Ok(token),
            other => Err(format!(
                "Token {:?} not found, found {:?}",
                token_kind, other
            )),
        }
    }
    fn match_keyword(&mut self, keyword: Keyword) -> Result<(), String> {
        let token = self.next_token();
        match &token.kind {
            TokenKind::Keyword(ref k) if k == &keyword => Ok(()),
            _ => Err(self.make_error(TokenKind::SemiColon, token)),
        }
    }
    fn match_operator(&mut self) -> Result<BinOp, String> {
        let token = self.next_token();
        BinOp::try_from(token.kind)
    }
    fn match_identifier(&mut self) -> Result<String, String> {
        match self.next_token().kind {
            TokenKind::Identifier(n) => Ok(n),
            other => Err(format!("Expected Identifier, found {:?}", other)),
        }
    }

    fn make_error(&mut self, token_kind: TokenKind, other: Token) -> String {
        match &self.raw {
            Some(raw_file) => format!(
                "Token {:?} not found, found {:?}\n{:?}",
                token_kind,
                other,
                highlight_position_in_file(raw_file.to_string(), other.to_owned().pos)
            ),
            None => format!("Token {:?} not found, found {:?}", token_kind, other),
        }
    }

    fn prev(&mut self) -> Option<Token> {
        self.prev.clone()
    }
}

impl Parser {
    fn parse_program(&mut self) -> Result<Program, String> {
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

    fn parse_function(&mut self) -> Result<Function, String> {
        self.match_keyword(Keyword::Function)?;
        let name = self.match_identifier()?;

        self.match_token(TokenKind::BraceOpen)?;

        let arguments: Vec<Variable> = match self.peek() {
            Some(t) if t.kind == TokenKind::BraceClose => Vec::new(),
            _ => self
                .parse_arguments()
                .expect("Failed to parse function arguments"),
        };

        self.match_token(TokenKind::BraceClose)?;
        self.match_token(TokenKind::CurlyBracesOpen)?;

        let mut statements = vec![];

        while let Err(_) = self.peek_token(TokenKind::CurlyBracesClose) {
            let statement = self.parse_statement()?;
            dbg!("{:?}", &statement);
            statements.push(statement);
        }

        self.match_token(TokenKind::CurlyBracesClose)?;

        Ok(Function {
            name: name,
            arguments: arguments,
            statements: statements,
        })
    }

    fn parse_arguments(&mut self) -> Result<Vec<Variable>, String> {
        let mut args = Vec::new();
        while let Err(_) = self.peek_token(TokenKind::BraceClose) {
            let next = self.next().ok_or_else(|| "Expected identifier")?;
            match next.kind {
                TokenKind::Identifier(name) => args.push(Variable { name: name }),
                _ => return Err(self.make_error(TokenKind::Identifier("Argument".into()), next)),
            }
        }

        Ok(args)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let token = self.peek().ok_or_else(|| "Expected token")?;
        let state = match &token.kind {
            TokenKind::Keyword(Keyword::Let) => self.parse_declare(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return(),
            TokenKind::Keyword(Keyword::If) => self.parse_conditional_statement(),
            TokenKind::Identifier(_) => {
                let ident = self.match_identifier()?;
                if let Ok(_) = self.peek_token(TokenKind::BraceOpen) {
                    let state = self.parse_function_call(Some(ident))?;
                    Ok(Statement::Exp(state))
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
            None => {
                self.next()
                    .ok_or_else(|| "Expected function identifier")?
                    .raw
            }
        };

        self.match_token(TokenKind::BraceOpen)?;

        let mut args = Vec::new();

        loop {
            let next = self.peek().ok_or_else(|| "Can not peek token")?;
            match &next.kind {
                TokenKind::BraceClose => break,
                TokenKind::Comma => {
                    self.next();
                    continue;
                }
                TokenKind::Identifier(_) | TokenKind::Literal(_) => {
                    args.push(self.parse_expression()?)
                }
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
        // TODO: Replace unwrap with make_error
        let peeked = self.peek().unwrap();
        match peeked.kind {
            TokenKind::SemiColon => Ok(Statement::Return(None)),
            _ => Ok(Statement::Return(Some(self.parse_expression()?))),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        let token = self.next_token();
        match token.kind {
            TokenKind::Literal(Value::Int) => {
                let state = match BinOp::try_from(self.peek().ok_or("Could not peek token")?.kind) {
                    Ok(_) => self.parse_bin_op(None)?,
                    Err(_) => Expression::Int(token.raw.parse::<u32>().map_err(|e| e.to_string())?),
                };
                Ok(state)
            }
            TokenKind::Literal(Value::Str) => {
                let state = match BinOp::try_from(self.peek().ok_or("Could not peek token")?.kind) {
                    Ok(_) => self.parse_bin_op(None)?,
                    Err(_) => Expression::Str(token.raw),
                };
                Ok(state)
            }
            TokenKind::Identifier(val) => {
                let next = self.peek().ok_or_else(|| "Token expected")?;
                let state = match &next.kind {
                    TokenKind::BraceOpen => {
                        let func_call = self.parse_function_call(Some(val))?;
                        match BinOp::try_from(self.peek().ok_or("Could not peek token")?.kind) {
                            Ok(_) => self.parse_bin_op(Some(func_call))?,
                            Err(_) => func_call,
                        }
                    }
                    _ => match BinOp::try_from(self.peek().ok_or("Could not peek token")?.kind) {
                        Ok(_) => self.parse_bin_op(Some(Expression::Variable(token.raw)))?,
                        Err(_) => Expression::Variable(val),
                    },
                };
                Ok(state)
            }
            other => Err(format!("Expected Expression, found {:?}", other)),
        }
    }

    fn parse_conditional_statement(&mut self) -> Result<Statement, String> {
        self.match_keyword(Keyword::If)?;
        let condition = self.parse_expression()?;
        self.match_token(TokenKind::CurlyBracesOpen)?;

        let mut statements = Vec::new();
        while let Err(_) = self.peek_token(TokenKind::CurlyBracesClose) {
            let statement = self.parse_statement()?;
            dbg!("{:?}", &statement);
            statements.push(statement);
        }

        self.match_token(TokenKind::CurlyBracesClose)?;

        match self.peek() {
            Some(tok) if tok.kind == TokenKind::Keyword(Keyword::Else) => {
                self.next_token();
                Ok(Statement::If(
                    condition,
                    statements,
                    Some(Box::new(self.parse_conditional_statement()?)),
                ))
            }
            _ => Ok(Statement::If(condition, statements, None)),
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
                    TokenKind::Identifier(_) | TokenKind::Literal(_) => {
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
        match (
            self.next_token().kind,
            self.peek().ok_or("Expected ; or =")?.kind,
        ) {
            (TokenKind::Identifier(name), TokenKind::SemiColon) => {
                Ok(Statement::Declare(Variable { name }, None))
            }
            (TokenKind::Identifier(name), TokenKind::Assign) => {
                self.drop(1);
                let exp = self.parse_expression().ok();
                Ok(Statement::Declare(Variable { name }, exp))
            }
            other => Err(format!("Expected identifier, found {:?}", other)),
        }
    }
}

pub fn parse(tokens: Vec<Token>, raw: Option<String>) -> Result<node_type::Program, String> {
    let mut parser = Parser::new(tokens, raw);

    parser.parse()
}
