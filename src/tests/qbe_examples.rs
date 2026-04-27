//! End-to-end integration tests for the QBE backend.
//!
//! For each example in `examples/`, this harness:
//!   1. Compiles the source through `sb --target qbe`
//!   2. Assembles the SSA via the `qbe` CLI
//!   3. Links the resulting `.s` with `builtin/builtin.c` via `cc`
//!   4. Runs the binary and asserts on stdout + exit code
//!
//! Tests are gated behind the presence of the `qbe` and `cc` binaries on
//! `$PATH`. If either is missing, the suite prints a notice and exits
//! successfully (skipping rather than failing).

use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn which(bin: &str) -> Option<PathBuf> {
    Command::new("which")
        .arg(bin)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
            } else {
                None
            }
        })
}

fn ensure_qbe_available() -> bool {
    if which("qbe").is_some() && which("cc").is_some() {
        true
    } else {
        eprintln!("[qbe_examples] skipping: `qbe` or `cc` is not on PATH");
        false
    }
}

fn build_compiler() {
    static BUILD: Once = Once::new();
    BUILD.call_once(|| {
        let status = Command::new("cargo")
            .args(["build", "--bin", "sb"])
            .current_dir(project_root())
            .status()
            .expect("cargo build failed to launch");
        assert!(status.success(), "cargo build failed");
    });
}

fn read_expected(name: &str) -> String {
    let path = project_root()
        .join("src/tests/qbe_examples/expected")
        .join(format!("{}.txt", name));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing golden output {}: {}", path.display(), e))
}

struct ExampleResult {
    stdout: String,
    exit: i32,
}

fn run_example(name: &str) -> Result<ExampleResult, String> {
    let root = project_root();
    let src = root.join("examples").join(format!("{}.sb", name));
    let tmp = std::env::temp_dir().join(format!("antimony-qbe-{}", name));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let ssa = tmp.join("out.ssa");
    let asm = tmp.join("out.s");
    let exe = tmp.join("out.exe");

    let sb = root.join("target/debug/sb");

    let compile = Command::new(&sb)
        .args([
            "--target", "qbe",
            "build",
            src.to_str().unwrap(),
            "-o", ssa.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("spawning sb: {}", e))?;
    if !compile.status.success() {
        return Err(format!(
            "sb compile failed for {}:\nstdout:{}\nstderr:{}",
            name,
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr),
        ));
    }

    let qbe = Command::new("qbe")
        .args(["-o", asm.to_str().unwrap(), ssa.to_str().unwrap()])
        .output()
        .map_err(|e| format!("spawning qbe: {}", e))?;
    if !qbe.status.success() {
        return Err(format!(
            "qbe failed for {}:\nstderr:{}",
            name,
            String::from_utf8_lossy(&qbe.stderr),
        ));
    }

    let cc = Command::new("cc")
        .args([
            asm.to_str().unwrap(),
            root.join("builtin/builtin.c").to_str().unwrap(),
            "-o", exe.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("spawning cc: {}", e))?;
    if !cc.status.success() {
        return Err(format!(
            "cc failed for {}:\nstderr:{}",
            name,
            String::from_utf8_lossy(&cc.stderr),
        ));
    }

    let run = Command::new(&exe)
        .output()
        .map_err(|e| format!("spawning binary: {}", e))?;
    Ok(ExampleResult {
        stdout: String::from_utf8_lossy(&run.stdout).into_owned(),
        exit: run.status.code().unwrap_or(-1),
    })
}

fn assert_example(name: &str) {
    if !ensure_qbe_available() {
        return;
    }
    build_compiler();
    let expected = read_expected(name);
    let result = run_example(name)
        .unwrap_or_else(|e| panic!("example {} failed pipeline: {}", name, e));
    assert_eq!(
        result.exit, 0,
        "{}: non-zero exit ({}); stdout was:\n{}",
        name, result.exit, result.stdout
    );
    assert_eq!(result.stdout, expected, "{}: stdout mismatch", name);
}

#[test] fn qbe_examples_hello_world() { assert_example("hello_world"); }
#[test] fn qbe_examples_fib()         { assert_example("fib"); }
#[test] fn qbe_examples_ackermann()   { assert_example("ackermann"); }
#[test] fn qbe_examples_greeter()     { assert_example("greeter"); }
#[test] fn qbe_examples_leapyear()    { assert_example("leapyear"); }
#[test] fn qbe_examples_loops()       { assert_example("loops"); }
#[test] fn qbe_examples_bubblesort()  { assert_example("bubblesort"); }
#[test] fn qbe_examples_sandbox()     { assert_example("sandbox"); }
