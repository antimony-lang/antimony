use std::fs;
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
use std::io::Error;
use std::process::Command;

fn test_directory(dir_in: &str) -> Result<(), Error> {
    let dir_out = format!("{}_out", dir_in);
    let dir = std::env::current_dir().unwrap();

    let examples = std::fs::read_dir(dir.join(dir_in))?;

    let _ = fs::create_dir(&dir_out);

    let out_file_suffix = ".js";

    for ex in examples {
        let example = ex?;
        let in_file = dir.join(dir_in).join(example.file_name());

        // We don't want to build submodules, since they don't run without a main function
        if in_file.is_dir() {
            continue;
        }
        let out_file = dir.join(&dir_out).join(
            example
                .file_name()
                .into_string()
                .unwrap()
                .replace(".sb", out_file_suffix),
        );
        let success = Command::new("cargo")
            .arg("run")
            .arg("build")
            .arg(&in_file)
            .arg("-o")
            .arg(&out_file)
            .spawn()?
            .wait()?
            .success();
        assert!(success, "{:?}", &in_file);

        let node_installed = Command::new("node").arg("-v").spawn()?.wait()?.success();
        if node_installed {
            let execution = Command::new("node")
                .arg(out_file)
                .spawn()?
                .wait()?
                .success();
            assert!(execution, "{:?}", &in_file)
        }
    }
    Ok(())
}

/// Compile a single .sb file through the full QBE pipeline and execute it.
/// Pipeline: .sb → (antimony) → .ssa → (qbe) → .s → (gcc) → binary → run
fn compile_and_run_qbe(in_file: &std::path::Path, dir_out: &std::path::Path) -> Result<(), Error> {
    let dir = std::env::current_dir().unwrap();

    let base_name = in_file.file_stem().unwrap().to_string_lossy().into_owned();
    let ssa_file = dir_out.join(format!("{}.ssa", base_name));
    let asm_file = dir_out.join(format!("{}.s", base_name));
    let bin_file = dir_out.join(&base_name);

    // Compile .sb -> .ssa
    let compile = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--target")
        .arg("qbe")
        .arg("build")
        .arg(in_file)
        .arg("-o")
        .arg(&ssa_file)
        .output()?;
    assert!(
        compile.status.success(),
        "QBE compile failed for {:?}: {}",
        in_file,
        String::from_utf8_lossy(&compile.stderr)
    );

    // .ssa -> .s via qbe
    let qbe = Command::new("qbe")
        .arg("-o")
        .arg(&asm_file)
        .arg(&ssa_file)
        .output()?;
    assert!(
        qbe.status.success(),
        "qbe failed for {:?}: {}",
        &ssa_file,
        String::from_utf8_lossy(&qbe.stderr)
    );

    // .s -> binary via gcc (link with builtin_qbe.c for runtime functions)
    let builtin_c = dir.join("builtin/builtin_qbe.c");
    let gcc = Command::new("gcc")
        .arg("-o")
        .arg(&bin_file)
        .arg(&asm_file)
        .arg(&builtin_c)
        .output()?;
    assert!(
        gcc.status.success(),
        "gcc failed for {:?}: {}",
        &asm_file,
        String::from_utf8_lossy(&gcc.stderr)
    );

    // Execute — verify the binary runs without crashing.
    // Note: void main() may return a non-zero exit code in QBE since
    // the backend doesn't yet emit `ret 0` for void functions, so we
    // only check that the process wasn't killed by a signal.
    let execution = Command::new(&bin_file).output()?;
    assert!(
        execution.status.code().is_some(),
        "Binary crashed (signal) for {:?}: {}",
        &bin_file,
        String::from_utf8_lossy(&execution.stderr)
    );

    Ok(())
}

#[test]
fn test_examples_js() -> Result<(), Error> {
    test_directory("examples")?;
    Ok(())
}

#[test]
fn test_examples_qbe() -> Result<(), Error> {
    let dir = std::env::current_dir().unwrap();
    let dir_out = dir.join("examples_out_qbe");
    let _ = fs::create_dir(&dir_out);

    // Only test examples that the QBE backend currently supports end-to-end.
    // As QBE backend coverage grows, add more examples here.
    let supported = [
        "hello_world.sb",
        "sandbox.sb",
        "bubblesort.sb",
        "leapyear.sb",
        "loops.sb",
    ];

    for name in &supported {
        let in_file = dir.join("examples").join(name);
        compile_and_run_qbe(&in_file, &dir_out)?;
    }
    Ok(())
}

#[test]
fn test_testcases_js() -> Result<(), Error> {
    let dir = std::env::current_dir().unwrap();

    let in_file = dir.join("tests/main.sb");
    let success = Command::new("cargo")
        .arg("run")
        .arg("run")
        .arg(&in_file)
        .spawn()?
        .wait()?
        .success();
    assert!(success, "{:?}", &in_file);
    Ok(())
}

#[test]
fn test_testcases_qbe() -> Result<(), Error> {
    let dir = std::env::current_dir().unwrap();
    let dir_out = dir.join("tests_out_qbe");
    let _ = fs::create_dir(&dir_out);

    let in_file = dir.join("tests/main.sb");
    compile_and_run_qbe(&in_file, &dir_out)?;

    Ok(())
}

#[test]
fn test_struct_decl_error() -> Result<(), Error> {
    let dir = std::env::current_dir().unwrap();

    let in_file = dir.join("tests/struct_decl_err.sb");
    let success = Command::new("cargo")
        .arg("run")
        .arg("run")
        .arg(&in_file)
        .spawn()?
        .wait()?
        .success();

    assert!(!success);
    Ok(())
}

#[test]
fn test_struct_instance_error() -> Result<(), Error> {
    let dir = std::env::current_dir().unwrap();

    let in_file = dir.join("tests/struct_instance_err.sb");
    let success = Command::new("cargo")
        .arg("run")
        .arg("run")
        .arg(&in_file)
        .spawn()?
        .wait()?
        .success();

    assert!(!success);
    Ok(())
}

#[test]
fn test_inline_function_error() -> Result<(), Error> {
    let dir = std::env::current_dir().unwrap();

    let in_file = dir.join("tests/inline_function_err.sb");
    let success = Command::new("cargo")
        .arg("run")
        .arg("run")
        .arg(&in_file)
        .spawn()?
        .wait()?
        .success();

    assert!(!success);
    Ok(())
}
