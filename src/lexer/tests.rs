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
use crate::lexer::*;

fn test_tokenize<F>(input: String, expected: F)
where
    F: Fn(FileId) -> Vec<Token>,
{
    let mut table = FileTable::new();
    let file = table.insert("<test>".into(), input);
    let tokens = tokenize(file, &table).unwrap();

    assert_eq!(tokens, expected(file));
}

fn test_tokenize_ignoring_whitespace<F>(input: String, expected: F)
where
    F: Fn(FileId) -> Vec<Token>,
{
    let mut table = FileTable::new();
    let file = table.insert("<test>".into(), input);
    let tokens = tokenize(file, &table)
        .unwrap()
        .into_iter()
        .filter(|token| token.kind != TokenKind::Whitespace)
        .collect::<Vec<Token>>();

    assert_eq!(tokens, expected(file));
}

#[test]
fn test_basic_tokenizing() {
    test_tokenize("1 = 2".to_owned(), |file| {
        vec![
            Token {
                len: 1,
                kind: TokenKind::Literal(Value::Int),
                raw: "1".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 1,
                },
            },
            Token {
                len: 1,
                kind: TokenKind::Whitespace,
                raw: " ".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 2,
                },
            },
            Token {
                len: 1,
                kind: TokenKind::Assign,
                raw: "=".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 3,
                },
            },
            Token {
                len: 1,
                kind: TokenKind::Whitespace,
                raw: " ".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 4,
                },
            },
            Token {
                len: 1,
                kind: TokenKind::Literal(Value::Int),
                raw: "2".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 5,
                },
            },
            Token {
                len: 0,
                kind: TokenKind::End,
                raw: "".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 6,
                },
            },
        ]
    });
}

#[test]
fn test_tokenizing_without_whitespace() {
    test_tokenize("1=2".to_owned(), |file| {
        vec![
            Token {
                len: 1,
                kind: TokenKind::Literal(Value::Int),
                raw: "1".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 1,
                },
            },
            Token {
                len: 1,
                kind: TokenKind::Assign,
                raw: "=".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 2,
                },
            },
            Token {
                len: 1,
                kind: TokenKind::Literal(Value::Int),
                raw: "2".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 3,
                },
            },
            Token {
                len: 0,
                kind: TokenKind::End,
                raw: "".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 4,
                },
            },
        ]
    });
}

#[test]
fn test_string() {
    test_tokenize_ignoring_whitespace("'aaa' \"bbb\"".to_owned(), |file| {
        vec![
            Token {
                len: 5,
                kind: TokenKind::Literal(Value::Str("aaa".into())),
                raw: "'aaa'".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 5,
                },
            },
            Token {
                len: 5,
                kind: TokenKind::Literal(Value::Str("bbb".into())),
                raw: "\"bbb\"".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 11,
                },
            },
            Token {
                len: 0,
                kind: TokenKind::End,
                raw: "".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 12,
                },
            },
        ]
    });
}

#[test]
fn test_string_markers_within_string() {
    test_tokenize_ignoring_whitespace("'\"aaa' \"'bbb\"".to_owned(), |file| {
        vec![
            Token {
                len: 6,
                kind: TokenKind::Literal(Value::Str("\"aaa".into())),
                raw: "'\"aaa'".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 6,
                },
            },
            Token {
                len: 6,
                kind: TokenKind::Literal(Value::Str("'bbb".into())),
                raw: "\"'bbb\"".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 13,
                },
            },
            Token {
                len: 0,
                kind: TokenKind::End,
                raw: "".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 14,
                },
            },
        ]
    });
}

#[test]
fn test_numbers() {
    test_tokenize("42".to_owned(), |file| {
        vec![
            Token {
                len: 2,
                kind: TokenKind::Literal(Value::Int),
                raw: "42".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 2,
                },
            },
            Token {
                len: 0,
                kind: TokenKind::End,
                raw: "".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 3,
                },
            },
        ]
    });
}

#[test]
fn test_binary_numbers() {
    test_tokenize("0b101010".to_owned(), |file| {
        vec![
            Token {
                len: 8,
                kind: TokenKind::Literal(Value::Int),
                raw: "0b101010".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 8,
                },
            },
            Token {
                len: 0,
                kind: TokenKind::End,
                raw: "".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 9,
                },
            },
        ]
    });
}

#[test]
fn test_octal_numbers() {
    test_tokenize("0o52".to_owned(), |file| {
        vec![
            Token {
                len: 4,
                kind: TokenKind::Literal(Value::Int),
                raw: "0o52".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 4,
                },
            },
            Token {
                len: 0,
                kind: TokenKind::End,
                raw: "".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 5,
                },
            },
        ]
    });
}

#[test]
fn test_hex_numbers() {
    test_tokenize("0x2A".to_owned(), |file| {
        vec![
            Token {
                len: 4,
                kind: TokenKind::Literal(Value::Int),
                raw: "0x2A".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 4,
                },
            },
            Token {
                len: 0,
                kind: TokenKind::End,
                raw: "".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 5,
                },
            },
        ]
    });
}

#[test]
fn test_functions() {
    test_tokenize_ignoring_whitespace("fn fib() {}".to_owned(), |file| {
        vec![
            Token {
                len: 2,
                kind: TokenKind::Keyword(Keyword::Function),
                raw: "fn".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 2,
                },
            },
            Token {
                len: 3,
                kind: TokenKind::Identifier("fib".into()),
                raw: "fib".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 6,
                },
            },
            Token {
                len: 1,
                kind: TokenKind::BraceOpen,
                raw: "(".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 7,
                },
            },
            Token {
                len: 1,
                kind: TokenKind::BraceClose,
                raw: ")".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 8,
                },
            },
            Token {
                len: 1,
                kind: TokenKind::CurlyBracesOpen,
                raw: "{".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 10,
                },
            },
            Token {
                len: 1,
                kind: TokenKind::CurlyBracesClose,
                raw: "}".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 11,
                },
            },
            Token {
                len: 0,
                kind: TokenKind::End,
                raw: "".to_owned(),
                pos: Position {
                    file,
                    line: 1,
                    column: 12,
                },
            },
        ]
    });
}

#[test]
fn test_comments() {
    test_tokenize_ignoring_whitespace(
        "// foo
fn fib() {}"
            .to_owned(),
        |file| {
            vec![
                Token {
                    len: 6,
                    kind: TokenKind::Comment,
                    raw: "// foo".to_owned(),
                    pos: Position {
                        file,
                        line: 1,
                        column: 6,
                    },
                },
                Token {
                    len: 2,
                    kind: TokenKind::Keyword(Keyword::Function),
                    raw: "fn".to_owned(),
                    pos: Position {
                        file,
                        line: 2,
                        column: 2,
                    },
                },
                Token {
                    len: 3,
                    kind: TokenKind::Identifier("fib".into()),
                    raw: "fib".to_owned(),
                    pos: Position {
                        file,
                        line: 2,
                        column: 6,
                    },
                },
                Token {
                    len: 1,
                    kind: TokenKind::BraceOpen,
                    raw: "(".to_owned(),
                    pos: Position {
                        file,
                        line: 2,
                        column: 7,
                    },
                },
                Token {
                    len: 1,
                    kind: TokenKind::BraceClose,
                    raw: ")".to_owned(),
                    pos: Position {
                        file,
                        line: 2,
                        column: 8,
                    },
                },
                Token {
                    len: 1,
                    kind: TokenKind::CurlyBracesOpen,
                    raw: "{".to_owned(),
                    pos: Position {
                        file,
                        line: 2,
                        column: 10,
                    },
                },
                Token {
                    len: 1,
                    kind: TokenKind::CurlyBracesClose,
                    raw: "}".to_owned(),
                    pos: Position {
                        file,
                        line: 2,
                        column: 11,
                    },
                },
                Token {
                    len: 0,
                    kind: TokenKind::End,
                    raw: "".to_owned(),
                    pos: Position {
                        file,
                        line: 2,
                        column: 12,
                    },
                },
            ]
        },
    );
}
