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
use crate::generator;
use crate::lexer;
use crate::parser;
use crate::Lib;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

pub fn build(in_file: &PathBuf, out_file: &PathBuf) -> Result<(), String> {
    let mut file = File::open(in_file).expect("Could not open file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Could not read file");
    let tokens = lexer::tokenize(&contents);
    let mut program = parser::parse(tokens, Some(contents))?;

    // C Backend currently does not support stdlib yet, since not all features are implemented
    if cfg!(feature = "backend_node") {
        let stdlib = build_stdlib();
        program.merge_with(stdlib);
    }

    let output = generator::generate(program);
    let mut file = std::fs::File::create(out_file).expect("create failed");
    file.write_all(output.as_bytes()).expect("write failed");
    file.flush().expect("Could not flush file");

    Ok(())
}

fn build_stdlib() -> parser::node_type::Program {
    let stdlib_raw =
        Lib::get("stdio.sb").expect("Standard library not found. This should not occur.");
    let stblib_str =
        std::str::from_utf8(&stdlib_raw).expect("Could not interpret standard library.");
    let stdlib_tokens = lexer::tokenize(&stblib_str);

    parser::parse(stdlib_tokens, Some(stblib_str.into())).expect("Could not parse stdlib")
}
