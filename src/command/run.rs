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
use std::process::Command;
use std::process::Stdio;

pub fn run(target: Target, in_file: PathBuf) -> Result<(), String> {
    let mut buf = Box::<Vec<u8>>::default();
    build::build_to_buffer(&target, &in_file, &mut buf)?;

    match target {
        Target::JS => {
            let mut process = Command::new("node")
                .stdin(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Could not spawn Node.js process: {}", e))?;

            process
                .stdin
                .as_ref()
                .unwrap()
                .write_all(&buf)
                .map_err(|e| format!("Could not write to Node.js process: {}", e))?;

            process.wait().unwrap();
        }
        _ => todo!(),
    }
    Ok(())
}
