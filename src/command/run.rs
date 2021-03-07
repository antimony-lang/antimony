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
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

pub fn run(target: Target, in_file: PathBuf) -> Result<(), String> {
    let mut buf = Box::new(Vec::new());
    build::build_to_buffer(target, &in_file, &mut buf)?;
    match target {
        Target::JS => {
            let process = match Command::new("node")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Err(why) => panic!("couldn't spawn wc: {}", why),
                Ok(process) => process,
            };

            &process
                .stdin
                .unwrap()
                .write_all(&buf)
                .map(|_| "Could not write to nodejs process".to_string());

            let mut s = Vec::new();
            process
                .stdout
                .unwrap()
                .read_to_end(&mut s)
                .expect("Could not read from child process");
            std::io::stdout()
                .write_all(&s)
                .expect("Could not write to stdout");
        }
        _ => todo!(),
    }
    Ok(())
}
