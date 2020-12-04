use crate::lexer::Position;
use std::str::Chars;

/// Peekable iterator over a char sequence.
///
/// Next characters can be peeked via `nth_char` method,
/// and position can be shifted forward via `bump` method.
pub(crate) struct Cursor<'a> {
    initial_length: usize,
    pos: &'a mut Position,
    len: usize,
    chars: Chars<'a>,
    prev: char,
}

pub(crate) const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    pub(crate) fn new(
        input: &'a str,
        initial_len: usize,
        position: &'a mut Position,
    ) -> Cursor<'a> {
        Cursor {
            initial_length: initial_len,
            len: input.len(),
            chars: input.chars(),
            #[cfg(debug_assertions)]
            prev: EOF_CHAR,
            pos: position,
        }
    }

    /// For debug assertions only
    /// Returns the last eaten symbol (or '\0' in release builds).
    pub(crate) fn prev(&self) -> char {
        #[cfg(debug_assertions)]
        {
            self.prev
        }

        #[cfg(not(debug_assertions))]
        {
            '\0'
        }
    }

    /// Returns nth character relative to the current cursor position.
    /// If requested position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file,
    /// it should be checked with `is_eof` method.
    fn nth_char(&self, n: usize) -> char {
        self.chars().nth(n).unwrap_or(EOF_CHAR)
    }

    /// Peeks the next symbol from the input stream without consuming it.
    pub(crate) fn first(&self) -> char {
        self.nth_char(0)
    }

    /// Checks if there is nothing more to consume.
    pub(crate) fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    /// Returns amount of already consumed symbols.
    pub(crate) fn len_consumed(&self) -> usize {
        self.len - self.chars.as_str().len()
    }

    /// Returns a `Chars` iterator over the remaining characters.
    pub(crate) fn chars(&self) -> Chars<'a> {
        self.chars.clone()
    }

    pub(crate) fn pos(&self) -> Position {
        let mut p = self.pos.clone();
        p
    }

    /// Moves to the next character.
    pub(crate) fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        // If first token, the position should be set to 0
        match self.pos.raw {
            usize::MAX => self.pos.raw = 0,
            _ => {
                self.pos.raw += 1;
                self.pos.offset += 1;
            }
        }

        if c == '\n' {
            self.pos.line += 1;
            self.pos.offset = 0;
        }

        #[cfg(debug_assertions)]
        {
            self.prev = c;
        }

        Some(c)
    }
}
