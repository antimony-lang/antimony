use crate::lexer::Token;

pub struct Parser {
    tokens: Box<dyn Iterator<Item = Token>>,
    current: Option<Token>,
    indentation_level: usize,
}

impl Parser {
    pub(crate) fn new(tokens: impl Iterator<Item = Token> + 'static) -> Self {
        Parser {
            tokens: Box::new(tokens),
            current: None,
            indentation_level: 0,
        }
    }

    fn next(&mut self) {
        self.current = self.tokens.next();
    }
}

#[derive(Debug)]
pub struct AST;

pub fn parse(tokens: impl Iterator<Item = Token> + 'static) -> AST {
    let mut parser = Parser::new(tokens);
    let ast = AST {};

    loop {
        parser.next();
        break;
    }

    ast
}
