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

use crate::Opt::Build;
use std::path::PathBuf;
use structopt::StructOpt;

mod command;
mod generator;
mod lexer;
mod parser;
#[cfg(test)]
mod tests;
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
        in_file: PathBuf,
        #[structopt(short, long)]
        out_file: PathBuf,
    },
}

fn main() -> Result<(), String> {
    let opts = Opt::from_args();

    match opts {
        Build { in_file, out_file } => command::build::build(in_file, out_file)?,
    };

    Ok(())
}
