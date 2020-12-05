use crate::lexer::Keyword;
use crate::lexer::{Token, TokenKind, Value};
use crate::parser::node_type::*;
use crate::util::string_util::highlight_position_in_file;
use std::iter::Peekable;
use std::vec::IntoIter;

mod node_type;

#[cfg(test)]
mod tests;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    peeked: Vec<Token>,
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
            raw: raw,
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        self.parse_program()
    }

    fn next(&mut self) -> Option<Token> {
        if self.peeked.is_empty() {
            self.tokens.next()
        } else {
            self.peeked.pop()
        }
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
            arguments: Vec::new(),
            statements: statements,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let token = self.next_token();
        dbg!(&token);
        match &token.kind {
            TokenKind::Keyword(Keyword::Let) => {
                let state = self.parse_declare();
                self.match_token(TokenKind::SemiColon)?;

                state
            }
            TokenKind::Keyword(Keyword::Return) => {
                let state = self.parse_return()?;
                self.match_token(TokenKind::SemiColon)?;

                Ok(state)
            }
            _ => Err(self.make_error(TokenKind::Unknown, token)),
        }
    }

    fn parse_return(&mut self) -> Result<Statement, String> {
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
                let state = Expression::Int(token.raw.parse::<u32>().map_err(|e| e.to_string())?);
                Ok(state)
            }
            TokenKind::Literal(Value::Str) => {
                let state = Expression::Str(token.raw);
                Ok(state)
            }
            TokenKind::Identifier(val) => {
                let state = Expression::Variable(val);
                Ok(state)
            }
            other => Err(format!("Expected Expression, found {:?}", other)),
        }
    }

    fn parse_declare(&mut self) -> Result<Statement, String> {
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
