extern crate structopt;

use crate::generator::Generator;
use crate::Opt::Build;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

mod generator;
mod lexer;
mod parser;
mod util;

#[derive(StructOpt, Debug)]
enum Opt {
    #[structopt()]
    Build {
        #[structopt(default_value = "./examples/playground.sb")]
        in_file: PathBuf,
        #[structopt(short, long, default_value = "./examples_out/playground.js")]
        out_file: PathBuf,
    },
}

fn main() -> Result<(), String> {
    let opts = Opt::from_args();

    let (in_file, out_file) = match opts {
        Build { in_file, out_file } => (in_file, out_file),
    };
    let mut file = File::open(in_file).expect("Could not open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let tokens = lexer::tokenize(&contents);
    // let ast = parser::parse(tokens.into_iter());

    let program = parser::parse(tokens, Some(contents))?;

    dbg!(":#?", &program);

    let output = generator::js::JsGenerator::generate(program);
    let mut file = std::fs::File::create(out_file).expect("create failed");
    file.write_all(output.as_bytes()).expect("write failed");
    file.flush().expect("Could not flush file");
    Ok(())
}
