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
use crate::Builtins;
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

fn run_qbe(buf: Vec<u8>, in_file: &Path, args: &[String]) -> Result<()> {
    let dir = std::env::temp_dir().join("antimony_qbe");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create temp directory: {}", e))?;

    let filename = in_file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?;

    let ssa_path = dir.join(format!("{}.ssa", filename));
    let asm_path = dir.join(format!("{}.s", filename));
    let exe_path = dir.join(format!("{}.exe", filename));
    let builtins_path = dir.join("builtin_qbe.c");

    // Write SSA file
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ssa_path)
        .map_err(|e| format!("Failed to open SSA file: {}", e))?
        .write_all(&buf)
        .map_err(|e| format!("Failed to write SSA file: {}", e))?;

    // Write C builtins file
    let raw_builtins = Builtins::get("builtin_qbe.c")
        .expect("Could not locate QBE builtin functions")
        .data;
    std::fs::write(&builtins_path, &*raw_builtins)
        .map_err(|e| format!("Failed to write builtins file: {}", e))?;

    // Compile and run
    run_command(Command::new("qbe").arg("-o").arg(&asm_path).arg(&ssa_path))?;
    run_command(
        Command::new("gcc")
            .arg(&builtins_path)
            .arg(&asm_path)
            .arg("-o")
            .arg(&exe_path),
    )?;
    let result = run_command(Command::new(&exe_path).args(args));
    let _ = std::fs::remove_dir_all(&dir); // best-effort cleanup
    result
}

pub fn run(target: Target, in_file: PathBuf, args: Vec<String>) -> Result<()> {
    let mut buf = Box::<Vec<u8>>::default();
    build::build_to_buffer(&target, &in_file, &mut buf)?;

    match target {
        Target::JS => run_node(&buf),
        Target::Qbe => run_qbe(*buf, &in_file, &args),
        _ => Err("Unsupported target".to_string()),
    }
}
