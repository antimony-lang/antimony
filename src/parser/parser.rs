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
use super::{Error, Result};
use crate::ast::*;
use crate::lexer::{Keyword, Token, TokenKind};
use std::convert::TryFrom;
use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    peeked: Vec<Token>,
    current: Option<Token>,
}

impl Parser {
    #[allow(clippy::needless_collect)] // TODO
    pub fn new(tokens: Vec<Token>) -> Parser {
        let tokens_without_whitespace: Vec<Token> = tokens
            .into_iter()
            .filter(|token| token.kind != TokenKind::Whitespace && token.kind != TokenKind::Comment)
            .collect();
        Parser {
            tokens: tokens_without_whitespace.into_iter().peekable(),
            peeked: vec![],
            current: None,
        }
    }

    pub fn parse(&mut self) -> Result<Module> {
        self.parse_module()
    }

    pub(super) fn next(&mut self) -> Result<Token> {
        let item = if self.peeked.is_empty() {
            self.tokens.next()
        } else {
            self.peeked.pop()
        };

        self.current = item.to_owned();
        match item {
            Some(Token {
                kind: TokenKind::End,
                pos,
                ..
            }) => Err(Error::new(
                pos,
                "Expected token, found End of file".to_owned(),
            )),
            Some(token) => Ok(token),
            None => unreachable!(),
        }
    }

    pub(super) fn peek(&mut self) -> Result<Token> {
        let token = self.next()?;
        self.push(token.to_owned());
        Ok(token)
    }

    pub(super) fn push(&mut self, token: Token) {
        self.peeked.push(token);
    }

    pub(super) fn has_more(&mut self) -> bool {
        if !self.peeked.is_empty() {
            return true;
        }
        match self.tokens.peek() {
            None => false,
            Some(Token {
                kind: TokenKind::End,
                ..
            }) => false,
            Some(_) => true,
        }
    }

    pub(super) fn match_token(&mut self, token_kind: TokenKind) -> Result<Token> {
        match self.next()? {
            token if token.kind == token_kind => Ok(token),
            other => Err(self.make_error(token_kind, other)),
        }
    }

    pub(super) fn peek_token(&mut self, token_kind: TokenKind) -> Result<Token> {
        match self.peek()? {
            token if token.kind == token_kind => Ok(token),
            other => Err(self.make_error(token_kind, other)),
        }
    }

    pub(super) fn match_keyword(&mut self, keyword: Keyword) -> Result<()> {
        let token = self.next()?;
        match &token.kind {
            TokenKind::Keyword(ref k) if k == &keyword => Ok(()),
            _ => Err(self.make_error(TokenKind::SemiColon, token)),
        }
    }

    pub(super) fn match_operator(&mut self) -> Result<BinOp> {
        let token = self.next()?;
        BinOp::try_from(token.kind.clone()).map_err(|err| Error::new(token.pos, err))
    }

    pub(super) fn match_identifier(&mut self) -> Result<String> {
        let token = self.next()?;
        match &token.kind {
            TokenKind::Identifier(n) => Ok(n.to_string()),
            other => Err(Error::new(
                token.pos,
                format!("Expected Identifier, found {:?}", other),
            )),
        }
    }

    pub(super) fn make_error(&mut self, token_kind: TokenKind, other: Token) -> Error {
        Error::new(
            other.pos,
            format!("Token {:?} not found, found {:?}", token_kind, other),
        )
    }
}
