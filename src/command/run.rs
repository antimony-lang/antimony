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
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

type Result<T> = std::result::Result<T, String>;

fn run_command(cmd: &mut Command) -> Result<()> {
    cmd.spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?
        .wait()
        .map_err(|e| format!("Failed to wait for process: {}", e))
        .map(|_| ())
}

fn run_node(buf: &[u8]) -> Result<()> {
    let process = Command::new("node")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not spawn Node.js process: {}", e))?;

    // Write to stdin
    process
        .stdin
        .ok_or("Failed to open stdin")?
        .write_all(buf)
        .map_err(|e| format!("Could not write to Node.js process: {}", e))?;

    // Read from stdout
    let mut output = Vec::new();
    process
        .stdout
        .ok_or("Failed to open stdout")?
        .read_to_end(&mut output)
        .map_err(|e| format!("Could not read from child process: {}", e))?;

    // Write to stdout
    std::io::stdout()
        .write_all(&output)
        .map_err(|e| format!("Could not write to stdout: {}", e))
}

fn run_qbe(buf: Vec<u8>, in_file: &Path) -> Result<()> {
    let dir_path = "./"; // TODO: Use this for changing build directory
    let filename = in_file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?;

    // Create paths without array destructuring
    let ssa_path = format!("{dir_path}{}.ssa", filename);
    let asm_path = format!("{dir_path}{}.s", filename);
    let exe_path = format!("{dir_path}{}.exe", filename);

    // Write SSA file
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ssa_path)
        .map_err(|e| format!("Failed to open SSA file: {}", e))?
        .write_all(&buf)
        .map_err(|e| format!("Failed to write SSA file: {}", e))?;

    // Compile and run
    run_command(Command::new("qbe").arg(&ssa_path).arg("-o").arg(&asm_path))?;
    run_command(Command::new("gcc").arg(&asm_path).arg("-o").arg(&exe_path))?;
    run_command(&mut Command::new(&exe_path))
}

pub fn run(target: Target, in_file: PathBuf) -> Result<()> {
    let mut buf = Box::<Vec<u8>>::default();
    build::build_to_buffer(&target, &in_file, &mut buf)?;

    match target {
        Target::JS => run_node(&buf),
        Target::Qbe => run_qbe(*buf, &in_file),
        _ => Err("Unsupported target".to_string()),
    }
}
