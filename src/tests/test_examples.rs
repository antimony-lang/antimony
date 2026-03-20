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

fn is_tool_available(tool: &str, version_arg: &str) -> bool {
    Command::new(tool)
        .arg(version_arg)
        .spawn()
        .map(|mut c| {
            let _ = c.wait();
            true
        })
        .unwrap_or(false)
}

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

fn test_directory_qbe(dir_in: &str) -> Result<(), Error> {
    if !is_tool_available("qbe", "-h") || !is_tool_available("gcc", "--version") {
        return Ok(());
    }

    let dir_out = format!("{}_out_qbe", dir_in);
    let dir = std::env::current_dir().unwrap();
    let examples = std::fs::read_dir(dir.join(dir_in))?;
    let _ = fs::create_dir(&dir_out);

    for ex in examples {
        let example = ex?;
        let in_file = dir.join(dir_in).join(example.file_name());

        if in_file.is_dir() {
            continue;
        }

        let stem = example
            .file_name()
            .into_string()
            .unwrap()
            .replace(".sb", "");
        let ssa_file = dir.join(&dir_out).join(format!("{}.ssa", stem));
        let asm_file = dir.join(&dir_out).join(format!("{}.s", stem));
        let exe_file = dir.join(&dir_out).join(&stem);

        let build_success = Command::new("cargo")
            .arg("run")
            .arg("build")
            .arg(&in_file)
            .arg("-o")
            .arg(&ssa_file)
            .spawn()?
            .wait()?
            .success();
        assert!(build_success, "QBE build failed: {:?}", &in_file);

        let qbe_success = Command::new("qbe")
            .arg(&ssa_file)
            .arg("-o")
            .arg(&asm_file)
            .spawn()?
            .wait()?
            .success();
        assert!(qbe_success, "qbe compile failed: {:?}", &ssa_file);

        let gcc_success = Command::new("gcc")
            .arg(&asm_file)
            .arg("-o")
            .arg(&exe_file)
            .spawn()?
            .wait()?
            .success();
        assert!(gcc_success, "gcc link failed: {:?}", &asm_file);

        let run_success = Command::new(&exe_file).spawn()?.wait()?.success();
        assert!(run_success, "execution failed: {:?}", &exe_file);
    }
    Ok(())
}

/// Examples that the C backend cannot yet handle due to missing type inference
/// or unsupported language features (e.g. printing arrays, for-each over
/// variables whose element type is not statically known).
const C_BACKEND_SKIP: &[&str] = &["bubblesort.sb", "loops.sb"];

fn test_directory_c(dir_in: &str) -> Result<(), Error> {
    if !is_tool_available("gcc", "--version") {
        return Ok(());
    }

    let dir_out = format!("{}_out_c", dir_in);
    let dir = std::env::current_dir().unwrap();
    let examples = std::fs::read_dir(dir.join(dir_in))?;
    let _ = fs::create_dir(&dir_out);

    for ex in examples {
        let example = ex?;
        let in_file = dir.join(dir_in).join(example.file_name());

        if in_file.is_dir() {
            continue;
        }

        // Skip examples with known C-backend limitations
        let filename = example.file_name().into_string().unwrap();
        if C_BACKEND_SKIP.contains(&filename.as_str()) {
            continue;
        }

        let stem = example
            .file_name()
            .into_string()
            .unwrap()
            .replace(".sb", "");
        let c_file = dir.join(&dir_out).join(format!("{}.c", stem));
        let exe_file = dir.join(&dir_out).join(&stem);

        let build_success = Command::new("cargo")
            .arg("run")
            .arg("build")
            .arg(&in_file)
            .arg("-o")
            .arg(&c_file)
            .spawn()?
            .wait()?
            .success();
        assert!(build_success, "C build failed: {:?}", &in_file);

        let gcc_success = Command::new("gcc")
            .arg(&c_file)
            .arg("-o")
            .arg(&exe_file)
            .spawn()?
            .wait()?
            .success();
        assert!(gcc_success, "gcc compile failed: {:?}", &c_file);

        let run_success = Command::new(&exe_file).spawn()?.wait()?.success();
        assert!(run_success, "execution failed: {:?}", &exe_file);
    }
    Ok(())
}

#[test]
fn test_examples() -> Result<(), Error> {
    test_directory("examples")?;
    Ok(())
}

#[test]
fn test_examples_qbe() -> Result<(), Error> {
    test_directory_qbe("examples")?;
    Ok(())
}

#[test]
fn test_examples_c() -> Result<(), Error> {
    test_directory_c("examples")?;
    Ok(())
}

#[test]
fn test_testcases() -> Result<(), Error> {
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
    if !is_tool_available("qbe", "-h") || !is_tool_available("gcc", "--version") {
        return Ok(());
    }

    let dir = std::env::current_dir().unwrap();
    let in_file = dir.join("tests/main.sb");
    let success = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--target")
        .arg("qbe")
        .arg("run")
        .arg(&in_file)
        .spawn()?
        .wait()?
        .success();
    assert!(success, "{:?}", &in_file);
    Ok(())
}

#[test]
fn test_testcases_c() -> Result<(), Error> {
    if !is_tool_available("gcc", "--version") {
        return Ok(());
    }

    let dir = std::env::current_dir().unwrap();
    let in_file = dir.join("tests/main.sb");
    let success = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--target")
        .arg("c")
        .arg("run")
        .arg(&in_file)
        .spawn()?
        .wait()?
        .success();
    assert!(success, "{:?}", &in_file);
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
