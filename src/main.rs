#![allow(unknown_lints)]

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
extern crate tempfile;

use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

mod ast;
mod builder;
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

#[derive(Debug)]
enum Target {
    C,
    JS,
    LLVM,
}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();

        match s.as_str() {
            #[cfg(feature = "backend_c")]
            "c" => Ok(Target::C),

            #[cfg(feature = "backend_node")]
            "js" => Ok(Target::JS),

            #[cfg(feature = "backend_llvm")]
            "llvm" => Ok(Target::LLVM),

            _ => Err(format!(
                "no target {T} found, maybe you forgot to enable backend_{T} feature?",
                T = s
            )),
        }
    }
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt()]
    Build {
        in_file: PathBuf,
        #[structopt(short, long)]
        out_file: PathBuf,
    },
    #[structopt()]
    Run { in_file: PathBuf },
}

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(subcommand)]
    command: Command,

    /// Target language. Options: c, js, llvm
    #[structopt(long, short, default_value = "js", parse(try_from_str))]
    target: Target,
}

fn main() -> Result<(), String> {
    let opts = Opt::from_args();

    match opts.command {
        Command::Build { in_file, out_file } => command::build::build(&in_file, &out_file)?,
        Command::Run { in_file } => command::run::run(in_file)?,
    };

    Ok(())
}
