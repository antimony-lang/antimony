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
use crate::command::build;
use crate::generator::Target;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::process::Stdio;
use tempfile::tempdir;

pub fn run(target: Target, in_file: PathBuf) -> Result<(), String> {
    let out_dir = tempdir()
        .expect("Could not create temporary file")
        .into_path();

    let intermediate_out_file_path = out_dir.join("intermediate.c");
    build::build(target, &in_file, &intermediate_out_file_path)?;
    let out_file = out_dir.join("out");
    match target {
        Target::C => {
            Command::new("/usr/bin/cc")
                .arg(&intermediate_out_file_path)
                .arg("-o")
                .arg(&out_file)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Could not spawn compilation process");

            let out = Command::new(out_file)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Could not spawn run process");

            std::io::stdout()
                .write_all(&out.stdout)
                .expect("Could not write to stdout");

            std::io::stderr()
                .write_all(&out.stderr)
                .expect("Could not write to stderr");
        }
        Target::JS => {
            let out = Command::new("node")
                .arg(&intermediate_out_file_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Could not spawn run process");

            std::io::stdout()
                .write_all(&out.stdout)
                .expect("Could not write to stdout");

            std::io::stderr()
                .write_all(&out.stderr)
                .expect("Could not write to stderr");

            process::exit(out.status.code().unwrap())
        }
        _ => todo!(),
    }
    Ok(())
}
