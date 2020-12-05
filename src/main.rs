use crate::generator::Generator;
use std::fs::File;
use std::io::Read;
use std::io::Write;

mod generator;
mod lexer;
mod parser;
mod util;

fn main() -> Result<(), String> {
    let mut file = File::open("examples/hello_world.sb").expect("Could not open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let tokens = lexer::tokenize(&contents);
    // let ast = parser::parse(tokens.into_iter());

    let program = parser::parse(tokens, Some(contents))?;

    dbg!(":#?", &program);

    let output = generator::js::JsGenerator::generate(program);
    let mut file = std::fs::File::create("examples_out/out.js").expect("create failed");
    file.write_all(output.as_bytes()).expect("write failed");
    file.flush().expect("Could not flush file");
    Ok(())
}
