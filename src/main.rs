use lexer::TokenKind;
use std::fs::File;
use std::io::Read;

mod lexer;
mod parser;

fn main() -> std::io::Result<()> {
    let mut file = File::open("examples/hello_world.fx")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let tokens = lexer::tokenize(&contents);
    // let ast = parser::parse(tokens.into_iter());

    for token in tokens {
        if token.kind != TokenKind::Whitespace && token.kind != TokenKind::CarriageReturn {
            println!("{:?}", token);
        }
    }
    Ok(())
}
