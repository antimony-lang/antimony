#[cfg(test)]
mod tests {
    use crate::lexer::*;

    #[test]
    fn test_basic_tokenizing() {
        let raw = tokenize("1 = 2");
        let mut tokens = raw.into_iter();

        assert_eq!(
            tokens.nth(0).unwrap(),
            Token {
                len: 1,
                kind: TokenKind::Literal(Value::Int),
                raw: "1".to_owned(),
                pos: 0
            }
        );

        assert_eq!(
            tokens.nth(0).unwrap(),
            Token {
                len: 1,
                kind: TokenKind::Whitespace,
                raw: " ".to_owned(),
                pos: 1
            }
        );

        assert_eq!(
            tokens.nth(0).unwrap(),
            Token {
                len: 1,
                kind: TokenKind::Assign,
                raw: "=".to_owned(),
                pos: 2
            }
        );

        assert_eq!(
            tokens.nth(0).unwrap(),
            Token {
                len: 1,
                kind: TokenKind::Whitespace,
                raw: " ".to_owned(),
                pos: 3
            }
        );

        assert_eq!(
            tokens.nth(0).unwrap(),
            Token {
                len: 1,
                kind: TokenKind::Literal(Value::Int),
                raw: "2".to_owned(),
                pos: 4
            }
        );
    }

    #[test]
    fn test_tokenizing_without_whitespace() {
        let mut tokens = tokenize("1=2").into_iter();

        assert_eq!(
            tokens.nth(0).unwrap(),
            Token {
                len: 1,
                kind: TokenKind::Literal(Value::Int),
                raw: "1".to_owned(),
                pos: 0
            }
        );

        assert_eq!(
            tokens.nth(0).unwrap(),
            Token {
                len: 1,
                kind: TokenKind::Assign,
                raw: "=".to_owned(),
                pos: 1
            }
        );

        assert_eq!(
            tokens.nth(0).unwrap(),
            Token {
                len: 1,
                kind: TokenKind::Literal(Value::Int),
                raw: "2".to_owned(),
                pos: 2
            }
        );
    }

    #[test]
    fn test_booleans() {
        let mut tokens = tokenize("true false").into_iter();

        assert_eq!(
            tokens.nth(0).unwrap(),
            Token {
                len: 4,
                kind: TokenKind::Keyword(Keyword::Boolean),
                raw: "true".to_owned(),
                pos: 0
            }
        );

        assert_eq!(
            tokens.nth(1).unwrap(),
            Token {
                len: 5,
                kind: TokenKind::Keyword(Keyword::Boolean),
                raw: "false".to_owned(),
                pos: 5
            }
        );
    }

    #[test]
    fn test_functions() {
        let mut tokens = tokenize("fn fib() {}").into_iter();

        assert_eq!(
            tokens.nth(0).unwrap(),
            Token {
                len: 2,
                kind: TokenKind::Keyword(Keyword::Function),
                raw: "fn".to_owned(),
                pos: 0
            }
        );
    }

    #[test]
    fn test_comments() {
        let mut tokens = tokenize(
            "
// foo
fn fib() {}
        ",
        )
        .into_iter()
        .filter(|t| {
            t.kind != TokenKind::Whitespace
                && t.kind != TokenKind::Tab
                && t.kind != TokenKind::CarriageReturn
        });

        assert_eq!(
            tokens.nth(0).unwrap(),
            Token {
                len: 6,
                kind: TokenKind::Comment,
                raw: "// foo".to_owned(),
                pos: 1
            }
        );

        assert_eq!(
            tokens.nth(0).unwrap(),
            Token {
                len: 2,
                kind: TokenKind::Keyword(Keyword::Function),
                raw: "fn".to_owned(),
                pos: 8
            }
        );
    }
}
