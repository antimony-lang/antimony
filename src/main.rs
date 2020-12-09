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
extern crate rust_embed;
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
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "lib/"]
pub struct Lib;

#[derive(RustEmbed)]
#[folder = "builtin/"]
pub struct Builtins;

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

    contents = append_stdlib(contents);

    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let tokens = lexer::tokenize(&contents);
    let program = parser::parse(tokens, Some(contents))?;

    let output = generator::js::JsGenerator::generate(program);
    let mut file = std::fs::File::create(out_file).expect("create failed");
    file.write_all(output.as_bytes()).expect("write failed");
    file.flush().expect("Could not flush file");
    Ok(())
}

fn append_stdlib(mut buffer: String) -> String {
    buffer += "// START stdlib\n\n";

    let stdlib = Lib::get("stdio.sb").expect("Standard library not found. This should not occur.");
    buffer += std::str::from_utf8(&stdlib).expect("Could not interpret standard library.");

    buffer += "\n// END stdlib\n\n";

    buffer
}
