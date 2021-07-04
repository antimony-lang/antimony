extern crate indextree;
extern crate lazy_static;
extern crate regex;
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

use generator::Target;
use std::path::PathBuf;
use std::process;
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

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt()]
    Build {
        in_file: PathBuf,
        /// Write output to a file. Use '-' to print to stdout
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

    /// Target language. Options: c, js, llvm, x86
    #[structopt(long, short, parse(try_from_str))]
    target: Option<Target>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let opts = Opt::from_args();

    match opts.command {
        Command::Build { in_file, out_file } => {
            let target = match opts.target {
                Some(t) => t,
                None => Target::from_extension(&out_file).ok_or_else(|| {
                    format!(
                        "Cannot detect target from output file {}, use --target option to set it explicitly",
                        &out_file.to_string_lossy(),
                    )
                })?,
            };

            command::build::build(&target, &in_file, &out_file)?
        }
        Command::Run { in_file } => command::run::run(opts.target.unwrap_or(Target::JS), in_file)?,
    };

    Ok(())
}
