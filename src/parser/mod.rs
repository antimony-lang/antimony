use crate::lexer::IdentifierKind;
use crate::lexer::{LiteralKind, Token, TokenKind};
use crate::parser::node_type::*;
use std::iter::Peekable;
use std::vec::IntoIter;

mod node_type;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    peeked: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens.into_iter().peekable(),
            peeked: vec![],
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
        loop {
            match self.peek().expect("Failed to peek token").kind {
                TokenKind::Whitespace | TokenKind::Tab | TokenKind::CarriageReturn => {
                    self.next_token();
                }
                _ => break,
            }
        }
        match self.next() {
            Some(token) if token.kind == token_kind => Ok(token),
            other => Err(format!(
                "Token {:?} not found, found {:?}",
                token_kind, other
            )),
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

    fn match_identifier_kind(&mut self, identifier_kind: IdentifierKind) -> Result<(), String> {
        let token = self.next_token();

        match token.kind {
            TokenKind::Identifier {
                kind: identifier_kind,
            } => Ok(()),
            other => Err(format!("Expected SemiColon, found {:?}", other)),
        }
    }

    fn match_identifier(&mut self) -> Result<String, String> {
        let token = self.next_token();

        // TODO: Match any IdentifierKind. This can definetely be prettier, but I couldn't figure it out in a hurry
        match &token.kind {
            TokenKind::Identifier {
                kind: IdentifierKind::Boolean,
            }
            | TokenKind::Identifier {
                kind: IdentifierKind::Else,
            }
            | TokenKind::Identifier {
                kind: IdentifierKind::Function,
            }
            | TokenKind::Identifier {
                kind: IdentifierKind::If,
            }
            | TokenKind::Identifier {
                kind: IdentifierKind::Let,
            }
            | TokenKind::Identifier {
                kind: IdentifierKind::Unknown,
            } => Ok(token.raw),
            other => Err(format!("Expected Identifier, found {:?}", other)),
        }
    }
}

impl Parser {
    fn parse_program(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();
        let globals = Vec::new();

        while self.has_more() {
            match self.next_token() {
                t if t.kind == TokenKind::Whitespace => continue,
                _ => functions.push(self.parse_function().expect("Failed to parse function")),
            }
        }

        Ok(Program {
            func: functions,
            globals: globals,
        })
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        self.match_identifier_kind(IdentifierKind::Function);
        let name = self
            .match_token(TokenKind::Literal {
                kind: LiteralKind::Str,
            })?
            .raw;

        self.match_token(TokenKind::BraceOpen);
        self.match_token(TokenKind::BraceClose);
        self.match_token(TokenKind::CurlyBracesOpen);
        self.match_token(TokenKind::CurlyBracesClose);

        Ok(Function {
            name: name,
            arguments: Vec::new(),
            statements: Vec::new(),
        })
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<node_type::Program, String> {
    let mut parser = Parser::new(tokens);

    parser.parse()
}
