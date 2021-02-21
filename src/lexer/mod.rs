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
pub(crate) mod cursor;

use self::TokenKind::*;
use cursor::Cursor;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub len: usize,
    pub raw: String,
    pub pos: Position,
}

impl Token {
    fn new(kind: TokenKind, len: usize, raw: String, pos: Position) -> Token {
        Token {
            kind,
            len,
            raw,
            pos,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub offset: usize,
    pub raw: usize,
}

/// Enum representing common lexeme types.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind {
    /// Any whitespace characters sequence.
    Whitespace,
    Identifier(String),
    Literal(Value),
    /// Keywords such as 'if' or 'else'
    Keyword(Keyword),
    /// // Lorem Ipsum
    Comment,
    /// "+"
    Plus,
    /// "-"
    Minus,
    /// "*"
    Star,
    /// "/"
    Slash,
    /// "%"
    Percent,
    /// ":"
    Colon,
    /// ";"
    SemiColon,
    /// "."
    Dot,
    /// "!"
    Exclamation,
    /// ","
    Comma,
    /// "="
    Assign,
    /// "=="
    Equals,
    /// "<"
    LessThan,
    /// "<="
    LessThanOrEqual,
    /// ">"
    GreaterThan,
    /// ">="
    GreaterThanOrEqual,
    /// "!="
    NotEqual,
    /// &&
    And,
    /// "||"
    Or,
    /// "+="
    PlusEqual,
    /// "-="
    MinusEqual,
    /// "*="
    StarEqual,
    /// "/="
    SlashEqual,
    /// "=>"
    ArrowRight,
    /// "("
    BraceOpen,
    /// ")"
    BraceClose,
    /// "["
    SquareBraceOpen,
    /// "]"
    SquareBraceClose,
    /// "{"
    CurlyBracesOpen,
    /// "}"
    CurlyBracesClose,
    /// "\t"
    Tab,
    /// "\n"
    CarriageReturn,
    /// Unknown token, not expected by the lexer, e.g. "№"
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Int,
    Str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Keyword {
    Let,
    If,
    Else,
    Return,
    While,
    For,
    In,
    Break,
    Continue,
    Function,
    Boolean,
    Struct,
    New,
    Match,
    Import,
    Unknown,
}

/// Creates an iterator that produces tokens from the input string.
pub fn tokenize(mut input: &str) -> Vec<Token> {
    let mut pos = Position {
        raw: usize::MAX,
        line: 1,
        offset: 0,
    };
    std::iter::from_fn(move || {
        if input.is_empty() {
            return None;
        }
        let token = first_token(input, &mut pos);
        input = &input[token.len..];
        Some(token)
    })
    .collect()
}

/// Parses the first token from the provided input string.
pub fn first_token(input: &str, pos: &mut Position) -> Token {
    debug_assert!(!input.is_empty());
    Cursor::new(input, pos).advance_token()
}

pub fn is_whitespace(c: char) -> bool {
    match c {
        ' ' | '\n' | '\r' | '\t' => true,
        '\u{00A0}' => {
            dbg!("Non-standard unicode character found: '\u{00A0}'");
            true
        }
        _ => false,
    }
}

/// True if `c` is valid as a first character of an identifier.
pub fn is_id_start(c: char) -> bool {
    ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == '_'
}

/// True if `c` is valid as a non-first character of an identifier.
pub fn is_id_continue(c: char) -> bool {
    ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || ('0'..='9').contains(&c) || c == '_'
}

impl Cursor<'_> {
    /// Parses a token from the input string.
    fn advance_token(&mut self) -> Token {
        // Original chars used to identify the token later on
        let original_chars = self.chars();
        // FIXME: Identical value, since it will be used twice and is not clonable later
        let original_chars2 = self.chars();
        let first_char = self.bump().unwrap();
        let token_kind = match first_char {
            c if is_whitespace(c) => self.whitespace(),
            '0'..='9' => self.number(),
            '"' | '\'' => self.string(),
            '.' => Dot,
            '+' => match self.first() {
                '=' => {
                    self.bump();
                    PlusEqual
                }
                _ => Plus,
            },
            '-' => match self.first() {
                '=' => {
                    self.bump();
                    MinusEqual
                }
                _ => Minus,
            },
            '*' => match self.first() {
                '=' => {
                    self.bump();
                    StarEqual
                }
                _ => Star,
            },
            '%' => Percent,
            '/' => match self.first() {
                '/' => {
                    self.bump();
                    self.comment()
                }
                '=' => {
                    self.bump();
                    SlashEqual
                }
                _ => Slash,
            },
            '=' => match self.first() {
                '=' => {
                    self.bump();
                    Equals
                }
                '>' => {
                    self.bump();
                    ArrowRight
                }
                _ => Assign,
            },
            ':' => Colon,
            ';' => SemiColon,
            ',' => Comma,
            '<' => match self.first() {
                '=' => {
                    self.bump();
                    LessThanOrEqual
                }
                _ => LessThan,
            },
            '>' => match self.first() {
                '=' => {
                    self.bump();
                    GreaterThanOrEqual
                }
                _ => GreaterThan,
            },
            '&' => match self.first() {
                '&' => {
                    self.bump();
                    And
                }
                _ => Unknown,
            },
            '|' => match self.first() {
                '|' => {
                    self.bump();
                    Or
                }
                _ => Unknown,
            },
            '!' => match self.first() {
                '=' => {
                    self.bump();
                    NotEqual
                }
                _ => Exclamation,
            },
            '(' => BraceOpen,
            ')' => BraceClose,
            '[' => SquareBraceOpen,
            ']' => SquareBraceClose,
            '{' => CurlyBracesOpen,
            '}' => CurlyBracesClose,
            c if is_id_start(c) => {
                let kind = self.identifier(c);
                if kind == Keyword::Unknown {
                    let mut ch: String = original_chars.collect();
                    ch.truncate(self.len_consumed());
                    TokenKind::Identifier(ch)
                } else {
                    TokenKind::Keyword(kind)
                }
            }
            '\n' => CarriageReturn,
            '\t' => Tab,
            _ => Unknown,
        };

        let len = self.len_consumed();
        let mut raw = original_chars2.collect::<String>();
        // Cut the original tokens to the length of the token
        raw.truncate(len);
        let position = self.pos();
        Token::new(token_kind, len, raw, position)
    }

    /// Eats symbols while predicate returns true or until the end of file is reached.
    /// Returns amount of eaten symbols.
    fn eat_while<F>(&mut self, mut predicate: F) -> usize
    where
        F: FnMut(char) -> bool,
    {
        let mut eaten: usize = 0;
        while predicate(self.first()) && !self.is_eof() {
            eaten += 1;
            self.bump();
        }

        eaten
    }

    fn whitespace(&mut self) -> TokenKind {
        debug_assert!(is_whitespace(self.prev()));
        self.eat_while(is_whitespace);
        Whitespace
    }

    fn number(&mut self) -> TokenKind {
        self.eat_digits();
        TokenKind::Literal(Value::Int)
    }

    fn string(&mut self) -> TokenKind {
        self.eat_string();

        TokenKind::Literal(Value::Str)
    }

    fn identifier(&mut self, first_char: char) -> Keyword {
        let mut original: String = self.chars().collect::<String>();
        let len = self.eat_while(is_id_continue);

        // Cut original "rest"-character stream to length of token
        // and prepend first character, because it has been eaten beforehand
        original.truncate(len);
        original = format!("{}{}", first_char, original);

        match original {
            c if c == "if" => Keyword::If,
            c if c == "else" => Keyword::Else,
            c if c == "fn" => Keyword::Function,
            c if c == "true" || c == "false" => Keyword::Boolean,
            c if c == "let" => Keyword::Let,
            c if c == "return" => Keyword::Return,
            c if c == "while" => Keyword::While,
            c if c == "for" => Keyword::For,
            c if c == "in" => Keyword::In,
            c if c == "break" => Keyword::Break,
            c if c == "continue" => Keyword::Continue,
            c if c == "struct" => Keyword::Struct,
            c if c == "new" => Keyword::New,
            c if c == "match" => Keyword::Match,
            c if c == "import" => Keyword::Import,
            _ => Keyword::Unknown,
        }
    }

    fn comment(&mut self) -> TokenKind {
        // FIXME: Might lead to a bug, if End of file is encountered
        while self.first() != '\n' {
            self.bump();
        }

        TokenKind::Comment
    }

    fn eat_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '_' => {
                    self.bump();
                }
                '0'..='9' => {
                    has_digits = true;
                    self.bump();
                }
                _ => break,
            }
        }
        has_digits
    }

    fn eat_string(&mut self) {
        // FIXME: double quoted strings could probably be ended by single quoted, and vice versa.
        // Possible fix: Pass the token of the string beginning down to this method and check against it.
        loop {
            match self.first() {
                '"' | '\'' => break,
                '\n' => panic!(
                    "String does not end on same line. At {}:{}",
                    self.pos().line,
                    self.pos().offset
                ),
                _ => self.bump(),
            };
        }

        // Eat last quote
        self.bump();
    }
}
