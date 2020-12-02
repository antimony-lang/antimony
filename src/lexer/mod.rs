pub(crate) mod cursor;

use self::TokenKind::*;
use cursor::Cursor;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub len: usize,
    pub raw: String,
}

impl Token {
    fn new(kind: TokenKind, len: usize, raw: String) -> Token {
        Token { kind, len, raw }
    }
}

/// Enum representing common lexeme types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind {
    /// Any whitespace characters sequence.
    Whitespace,
    Literal {
        kind: LiteralKind,
    },
    /// Keywords such as 'if' or 'else'
    Identifier {
        kind: IdentifierKind,
    },
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
    /// ":"
    Colon,
    /// "="
    Equals,
    /// "=="
    DeepEquals,
    /// "<"
    SmallerThen,
    /// ">"
    LargerThen,
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
pub enum LiteralKind {
    Int,
    Str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentifierKind {
    Let,
    If,
    Else,
    Function,
    Boolean,
    Unknown,
}

/// Creates an iterator that produces tokens from the input string.
pub fn tokenize(mut input: &str) -> Vec<Token> {
    std::iter::from_fn(move || {
        if input.is_empty() {
            return None;
        }
        let token = first_token(input);
        input = &input[token.len..];
        Some(token)
    })
    .collect()
}

/// Parses the first token from the provided input string.
pub fn first_token(input: &str) -> Token {
    debug_assert!(!input.is_empty());
    Cursor::new(input).advance_token()
}

pub fn is_whitespace(c: char) -> bool {
    match c {
        ' ' => true,
        _ => false,
    }
}

/// True if `c` is valid as a first character of an identifier.
/// See [Rust language reference](https://doc.rust-lang.org/reference/identifiers.html) for
/// a formal definition of valid identifier name.
pub fn is_id_start(c: char) -> bool {
    ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z') || c == '_'
}

/// True if `c` is valid as a non-first character of an identifier.
/// See [Rust language reference](https://doc.rust-lang.org/reference/identifiers.html) for
/// a formal definition of valid identifier name.
pub fn is_id_continue(c: char) -> bool {
    ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z') || ('0' <= c && c <= '9') || c == '_'
}

impl Cursor<'_> {
    /// Parses a token from the input string.
    fn advance_token(&mut self) -> Token {
        // Original chars used to identify the token later on
        let original_chars = self.chars();
        let first_char = self.bump().unwrap();
        let token_kind = match first_char {
            c if is_whitespace(c) => self.whitespace(),
            '0'..='9' => {
                let kind = self.number();

                TokenKind::Literal { kind }
            }
            '"' | '\'' => {
                let kind = self.string();

                TokenKind::Literal { kind }
            }
            '+' => Plus,
            '-' => Minus,
            '*' => Star,
            '/' => match self.first() {
                '/' => self.comment(),
                _ => Slash,
            },
            '=' => match self.first() {
                '=' => DeepEquals,
                _ => Equals,
            },
            ':' => Colon,
            '<' => SmallerThen,
            '>' => LargerThen,
            '(' => BraceOpen,
            ')' => BraceClose,
            '[' => SquareBraceOpen,
            ']' => SquareBraceClose,
            '{' => CurlyBracesOpen,
            '}' => CurlyBracesClose,
            c if is_id_start(c) => {
                let kind = self.identifier(c);

                Identifier { kind }
            }
            '\n' => CarriageReturn,
            '\t' => Tab,
            _ => Unknown,
        };

        let len = self.len_consumed();
        let mut raw = original_chars.collect::<String>();
        // Cut the original tokens to the length of the token
        raw.truncate(len);
        Token::new(token_kind, len, raw)
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

    fn number(&mut self) -> LiteralKind {
        self.eat_digits();
        LiteralKind::Int
    }

    fn string(&mut self) -> LiteralKind {
        self.eat_string();

        LiteralKind::Str
    }

    fn identifier(&mut self, first_char: char) -> IdentifierKind {
        let mut original: String = self.chars().collect::<String>();
        let len = self.eat_while(is_id_continue);

        // Cut original "rest"-character stream to length of token
        // and prepend first character, because it has been eaten beforehand
        original.truncate(len);
        original = format!("{}{}", first_char, original);

        match original {
            c if c == "if" => IdentifierKind::If,
            c if c == "else" => IdentifierKind::Else,
            c if c == "fn" => IdentifierKind::Function,
            c if c == "true" || c == "false" => IdentifierKind::Boolean,
            c if c == "let" => IdentifierKind::Let,
            _ => IdentifierKind::Unknown,
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
                _ => self.bump(),
            };
        }

        // Eat last quote
        self.bump();
    }
}
