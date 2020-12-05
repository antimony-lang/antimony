use crate::generator::Generator;
use std::fs::File;
use std::io::Read;

mod generator;
mod lexer;
mod parser;
mod util;

fn main() -> std::io::Result<()> {
    let mut file = File::open("examples/hello_world.sb")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let tokens = lexer::tokenize(&contents);
    // let ast = parser::parse(tokens.into_iter());

    let program = parser::parse(tokens, Some(contents));

    match program {
        Ok(p) => println!("{}", generator::x86::X86Generator::generate(p)),
        Err(e) => panic!(e),
    }

    Ok(())
}
