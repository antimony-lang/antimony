# QBE Backend Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every program in `examples/` compile, assemble, link, run, and produce the correct output through the QBE backend, locked in by an end-to-end integration test that runs `sb → qbe → cc → ./binary` for every example.

**Architecture:** Phased TDD. Each phase first lands a *failing* end-to-end test for one or more examples, then fixes the smallest set of issues required to flip that test green. The integration test harness is built first (Phase 0) so every subsequent task is a TDD loop — write the test, make it fail, fix it, verify green, commit.

**Tech Stack:**
- Rust 1.x (compiler) — `src/generator/qbe.rs`, `src/parser/infer.rs`, `src/builder/mod.rs`, `src/command/run.rs`
- `qbe` crate v3.0.0 (SSA emitter)
- `qbe` CLI (assembler), `cc` (linker), Antimony stdlib in `lib/`, C builtins in `builtin/builtin.c`
- Integration tests in `src/tests/test_examples.rs`, unit tests in `src/generator/tests/qbe_tests.rs`

**Out of scope (future plan):** struct methods/`Selff`, top-level globals, separate `tmp_counter` for labels, private linkage for module-internal symbols, replacing the unsafe `transmute` in `generate_array`, `Statement::Match`/x86 backend, LLVM target.

---

## Working Conventions

- **Branch / worktree:** this plan runs in `/Users/garrit/.cursor/worktrees/antimony/wio0` (already a worktree). All work commits onto whatever branch this worktree currently has checked out.
- **Required tooling:** `cargo`, `qbe` (Homebrew: `brew install qbe`), `cc`. The integration tests gracefully skip when `qbe` is not on `$PATH`.
- **Run a single example through the backend manually:**
  ```bash
  cargo run -- --target qbe build examples/hello_world.sb -o /tmp/hw.ssa
  qbe -o /tmp/hw.s /tmp/hw.ssa && cc /tmp/hw.s builtin/builtin.c -o /tmp/hw.exe && /tmp/hw.exe
  echo "exit=$?"
  ```
- **Run the full QBE integration suite:**
  ```bash
  cargo test --test '*' qbe_examples -- --nocapture
  ```
  (Test name pattern lands in Phase 0 / Task 1.)
- **Commit format:** Conventional commits — `feat(qbe): …`, `fix(qbe): …`, `test(qbe): …`, `chore(qbe): …`.

---

## File Structure

Files this plan creates or modifies. Each line states the file's responsibility after the plan lands.

**Create:**
- `src/tests/qbe_examples.rs` — End-to-end QBE integration test harness. Builds, assembles, links, runs each `examples/*.sb` and asserts on stdout + exit code. Skips cleanly if `qbe` binary is missing.
- `src/tests/qbe_examples/expected/<name>.txt` — Golden stdout for each example (one file per example).

**Modify:**
- `src/tests/mod.rs` — register the new `qbe_examples` test module.
- `src/generator/qbe.rs` — the QBE backend itself (most fixes land here).
- `src/parser/infer.rs` — extend `infer_expression` to cover `BinOp`.
- `src/builder/mod.rs` — load Antimony stdlib for the QBE target (currently JS-only).
- `src/generator/c.rs` *(optional)* — embed `builtin.c` already; we'll re-use the same mechanism.
- `src/command/run.rs` — fix `run_qbe`: use `cc` not `gcc`, link `builtin.c`, write intermediates to a temp dir.
- `builtin/builtin.c` — add missing `#include <stdlib.h>`, add `_strcat` and `_int_to_str` runtime helpers.
- `lib/io.sb` — add `println_int` / overloads (or a lowering-level helper for non-string `println`).
- `lib/array.sb` — fix `len()` to use the array's stored length header instead of scanning for a falsy element.

---

## Phase 0 — Test Scaffolding (must come first)

This phase lands a single test that fails for every example, so all later tasks have an objective signal.

### Task 1: End-to-end QBE example test harness

**Files:**
- Create: `src/tests/qbe_examples.rs`
- Create: `src/tests/qbe_examples/expected/hello_world.txt`
- Create: `src/tests/qbe_examples/expected/fib.txt`
- Create: `src/tests/qbe_examples/expected/ackermann.txt`
- Create: `src/tests/qbe_examples/expected/greeter.txt`
- Create: `src/tests/qbe_examples/expected/leapyear.txt`
- Create: `src/tests/qbe_examples/expected/loops.txt`
- Create: `src/tests/qbe_examples/expected/bubblesort.txt`
- Create: `src/tests/qbe_examples/expected/sandbox.txt`
- Modify: `src/tests/mod.rs`

- [ ] **Step 1: Add the harness module to `src/tests/mod.rs`**

```rust
mod test_examples;
mod qbe_examples;
```

- [ ] **Step 2: Create golden-output files**

`src/tests/qbe_examples/expected/hello_world.txt`:
```
Hello World
```

`src/tests/qbe_examples/expected/fib.txt`:
```
55
```

`src/tests/qbe_examples/expected/ackermann.txt`:
```
61
```

`src/tests/qbe_examples/expected/greeter.txt`:
```
Hello World
```

`src/tests/qbe_examples/expected/leapyear.txt`:
```
Leap year
```

`src/tests/qbe_examples/expected/loops.txt`:
```
One
Two
Three
Apple
Strawberry
Orange
```

`src/tests/qbe_examples/expected/bubblesort.txt`:
```
[1, 2, 3, 4, 5]
```

`src/tests/qbe_examples/expected/sandbox.txt`:
(empty — `sandbox.sb` returns 0 silently)

- [ ] **Step 3: Write the failing harness**

Create `src/tests/qbe_examples.rs`:

```rust
//! End-to-end integration tests for the QBE backend.
//!
//! For each example in `examples/`, this harness:
//!   1. Compiles the source through `sb --target qbe`
//!   2. Assembles the SSA via the `qbe` CLI
//!   3. Links the resulting `.s` with `builtin/builtin.c` via `cc`
//!   4. Runs the binary and asserts on stdout + exit code
//!
//! Tests are gated behind the presence of the `qbe` binary on `$PATH`.
//! If it's missing, the suite prints a notice and exits successfully.

use std::path::{Path, PathBuf};
use std::process::Command;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn which(bin: &str) -> Option<PathBuf> {
    Command::new("which")
        .arg(bin)
        .output()
        .ok()
        .and_then(|o| if o.status.success() { Some(PathBuf::from(String::from_utf8_lossy(&o.stdout).trim())) } else { None })
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
    let status = Command::new("cargo")
        .args(["build", "--bin", "sb"])
        .current_dir(project_root())
        .status()
        .expect("cargo build failed to launch");
    assert!(status.success(), "cargo build failed");
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
    if !ensure_qbe_available() { return; }
    build_compiler();
    let expected = read_expected(name);
    let result = run_example(name).unwrap_or_else(|e| panic!("example {} failed pipeline: {}", name, e));
    assert_eq!(result.exit, 0, "{}: non-zero exit ({}); stdout was:\n{}", name, result.exit, result.stdout);
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
```

- [ ] **Step 4: Run the harness — confirm it fails for every example**

Run: `cargo test --test sb qbe_examples -- --nocapture`

Wait — these tests live inside the `sb` binary's own test tree (because `src/tests/mod.rs` is included via `src/main.rs`). Use:

```bash
cargo test qbe_examples -- --nocapture
```

Expected: 1/8 might accidentally pass (`hello_world` exit-code-wise is currently 12, so it fails); the other 7 fail. This is the baseline.

- [ ] **Step 5: Commit**

```bash
git add src/tests/qbe_examples.rs src/tests/qbe_examples/ src/tests/mod.rs
git commit -m "test(qbe): add end-to-end integration harness for examples"
```

---

## Phase 1 — Make `hello_world` Pass End-to-End

Goal: `qbe_examples_hello_world` is green.

### Task 2: Fix `builtin/builtin.c` missing stdlib include

**Files:**
- Modify: `builtin/builtin.c`

- [ ] **Step 1: Reproduce the current link failure**

Run:
```bash
cc builtin/builtin.c -o /tmp/builtin_test.o -c
```
Expected: error `call to undeclared library function 'exit'`.

- [ ] **Step 2: Add the missing include**

`builtin/builtin.c` (full new content):
```c
/* START builtins */
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

void _printf(char *msg)
{
    printf("%s", msg);
}

void _exit(int code)
{
    exit(code);
}

/* END builtins */
```

- [ ] **Step 3: Verify it compiles**

Run: `cc builtin/builtin.c -c -o /tmp/builtin_test.o`
Expected: exit 0, no errors.

- [ ] **Step 4: Commit**

```bash
git add builtin/builtin.c
git commit -m "fix(builtin): add missing <stdlib.h> include for exit()"
```

### Task 3: Load Antimony stdlib for the QBE target

**Files:**
- Modify: `src/builder/mod.rs:64-67`

- [ ] **Step 1: Identify the call site**

Current code (`src/builder/mod.rs`):
```rust
// Append standard library
if matches!(target, Target::JS) {
    self.build_stdlib()?;
}
```

- [ ] **Step 2: Extend the gate to QBE (and C while we're here)**

Replace with:
```rust
// Append standard library for backends that don't already inline a stdlib
if matches!(target, Target::JS | Target::Qbe | Target::C) {
    self.build_stdlib()?;
}
```

- [ ] **Step 3: Confirm `bubblesort` now type-resolves `len()`**

Run: `cargo run -- --target qbe build examples/bubblesort.sb -o /tmp/bs.ssa`

Expected: previous error `Type of n could not be infered: FunctionCall { fn_name: "len", ... }` is gone (it may now fail later for a different reason — that's fine, fixed in later tasks).

- [ ] **Step 4: Confirm `hello_world` still emits SSA**

Run: `cargo run -- --target qbe build examples/hello_world.sb -o /tmp/hw.ssa && head /tmp/hw.ssa`
Expected: SSA contains both `$main` and `$println` definitions.

- [ ] **Step 5: Commit**

```bash
git add src/builder/mod.rs
git commit -m "feat(builder): load Antimony stdlib for QBE and C targets"
```

### Task 4: Make `main()` always terminate with exit 0 when source `main` is void

**Files:**
- Modify: `src/generator/qbe.rs` (function `generate_function`)

- [ ] **Step 1: Write a failing unit test**

Add to `src/generator/tests/qbe_tests.rs` (inside `mod tests`):
```rust
#[test]
fn test_void_main_returns_zero() {
    let func = create_function("main", None, create_block_stmt(vec![]));
    let module = create_module(vec![func], Vec::new());
    let result = QbeGenerator::generate(module).unwrap();
    let result_norm = normalize_qbe(&result);
    assert!(
        result_norm.contains("export function w $main()"),
        "main should be declared returning w (int):\n{}",
        result_norm
    );
    assert!(
        result_norm.contains("ret 0"),
        "void main should explicitly `ret 0`:\n{}",
        result_norm
    );
}
```

- [ ] **Step 2: Run it — verify it fails**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_void_main_returns_zero -- --nocapture`
Expected: FAIL — current output is `export function $main() { @start ret }` (no return type, bare `ret`).

- [ ] **Step 3: Modify `generate_function`**

In `src/generator/qbe.rs`, locate `generate_function` (currently around line 170). Replace the return-type / fall-through-return block with:

```rust
// Special-case `main`: always returns int and exits with 0 when the source
// declared no return type.
let is_main = func.name == "main";

let return_ty = if is_main {
    Some(qbe::Type::Word)
} else if let Some(ty) = &func.ret_type {
    Some(self.get_type(ty.to_owned())?.into_abi())
} else {
    None
};

let mut qfunc = qbe::Function::new(
    qbe::Linkage::public(),
    func.name.clone(),
    arguments,
    return_ty,
);

qfunc.add_block("start".to_owned());

self.generate_statement(&mut qfunc, &func.body)?;

let returns = qfunc.blocks.last().is_some_and(|b| {
    b.items.last().is_some_and(|item| {
        matches!(
            item,
            qbe::BlockItem::Statement(qbe::Statement::Volatile(qbe::Instr::Ret(_)))
        )
    })
});

if !returns {
    if is_main {
        qfunc.add_instr(qbe::Instr::Ret(Some(qbe::Value::Const(0))));
    } else if func.ret_type.is_none() {
        qfunc.add_instr(qbe::Instr::Ret(None));
    } else {
        return Err(format!(
            "Function '{}' does not return in all code paths",
            &func.name
        ));
    }
}
```

- [ ] **Step 4: Run unit test — verify it passes**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_void_main_returns_zero`
Expected: PASS.

- [ ] **Step 5: Run integration test for `hello_world`**

```bash
cargo test qbe_examples_hello_world -- --nocapture
```
Expected: PASS (stdout `Hello World\n`, exit 0).

- [ ] **Step 6: Commit**

```bash
git add src/generator/qbe.rs src/generator/tests/qbe_tests.rs
git commit -m "fix(qbe): emit 'ret 0' for void main so exit code is deterministic"
```

### Task 5: Clean up `run_qbe` to actually link the builtins and use a temp dir

**Files:**
- Modify: `src/command/run.rs:62-89`

- [ ] **Step 1: Manually reproduce the bug**

Run: `cargo run -- --target qbe run examples/hello_world.sb`
Expected: leaves `hello_world.ssa`, `hello_world.s`, `hello_world.exe` in the *current working directory* and segfaults / fails with undefined symbols if `gcc` is not installed.

- [ ] **Step 2: Replace `run_qbe`**

In `src/command/run.rs`, replace the `run_qbe` function with:

```rust
fn run_qbe(buf: Vec<u8>, in_file: &Path) -> Result<()> {
    use std::env;
    use tempdir_lite::tempdir; // see step 3 if no tempdir crate is available

    let filename = in_file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?;

    let tmp = env::temp_dir().join(format!("antimony-qbe-{}", filename));
    std::fs::create_dir_all(&tmp).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let ssa_path = tmp.join(format!("{}.ssa", filename));
    let asm_path = tmp.join(format!("{}.s", filename));
    let exe_path = tmp.join(format!("{}.exe", filename));

    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ssa_path)
        .map_err(|e| format!("Failed to open SSA file: {}", e))?
        .write_all(&buf)
        .map_err(|e| format!("Failed to write SSA file: {}", e))?;

    run_command(Command::new("qbe").arg(&ssa_path).arg("-o").arg(&asm_path))?;

    // Locate builtin.c relative to the running binary's CARGO_MANIFEST_DIR
    let builtin_c = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("builtin/builtin.c");
    let cc = env::var("CC").unwrap_or_else(|_| "cc".to_string());

    run_command(
        Command::new(&cc)
            .arg(&asm_path)
            .arg(&builtin_c)
            .arg("-o")
            .arg(&exe_path),
    )?;
    run_command(&mut Command::new(&exe_path))
}
```

- [ ] **Step 3: We don't actually need a tempdir crate; std::env::temp_dir is fine**

Remove the `use tempdir_lite::tempdir;` line — it isn't used. Final imports at the top of `src/command/run.rs` should already include `use std::path::{Path, PathBuf};`. Add `use std::path::PathBuf;` if missing.

- [ ] **Step 4: Verify `cargo run -- --target qbe run examples/hello_world.sb` works**

```bash
cargo run -- --target qbe run examples/hello_world.sb
```
Expected output: `Hello World` on stdout, exit 0, **no `*.ssa`/`*.s`/`*.exe` files left in the project root**.

- [ ] **Step 5: Commit**

```bash
git add src/command/run.rs
git commit -m "fix(run): use cc, link builtin.c, place QBE intermediates in temp dir"
```

---

## Phase 2 — Type Inference

Goal: enable the QBE backend to compile examples whose `let` statements depend on inferred types from binary expressions.

### Task 6: Infer types for binary expressions

**Files:**
- Modify: `src/parser/infer.rs`

- [ ] **Step 1: Write failing unit tests**

Append to `src/parser/infer.rs` (inside the file, after the existing functions):

```rust
#[cfg(test)]
mod inference_tests {
    use super::*;
    use crate::ast::hast::{HBinOp, HExpression};

    fn empty_table() -> SymbolTable {
        SymbolTable::new()
    }

    #[test]
    fn infers_arithmetic_as_int() {
        let expr = HExpression::BinOp {
            lhs: Box::new(HExpression::Int(2)),
            op: HBinOp::Addition,
            rhs: Box::new(HExpression::Int(3)),
        };
        assert_eq!(infer_expression(&expr, &empty_table()), Some(Type::Int));
    }

    #[test]
    fn infers_modulus_as_int() {
        let expr = HExpression::BinOp {
            lhs: Box::new(HExpression::Variable("year".into())),
            op: HBinOp::Modulus,
            rhs: Box::new(HExpression::Int(4)),
        };
        assert_eq!(infer_expression(&expr, &empty_table()), Some(Type::Int));
    }

    #[test]
    fn infers_comparison_as_bool() {
        let expr = HExpression::BinOp {
            lhs: Box::new(HExpression::Int(1)),
            op: HBinOp::Equal,
            rhs: Box::new(HExpression::Int(0)),
        };
        assert_eq!(infer_expression(&expr, &empty_table()), Some(Type::Bool));
    }

    #[test]
    fn infers_logical_as_bool() {
        let expr = HExpression::BinOp {
            lhs: Box::new(HExpression::Bool(true)),
            op: HBinOp::And,
            rhs: Box::new(HExpression::Bool(false)),
        };
        assert_eq!(infer_expression(&expr, &empty_table()), Some(Type::Bool));
    }

    #[test]
    fn infers_string_concat_as_str() {
        let expr = HExpression::BinOp {
            lhs: Box::new(HExpression::Str("Hello ".into())),
            op: HBinOp::Addition,
            rhs: Box::new(HExpression::Variable("name".into())),
        };
        assert_eq!(infer_expression(&expr, &empty_table()), Some(Type::Str));
    }
}
```

- [ ] **Step 2: Run the new tests — verify they all fail with `None`**

Run: `cargo test --lib parser::infer::inference_tests -- --nocapture`
Expected: 5 failures, each "expected Some(...) got None".

- [ ] **Step 3: Extend `infer_expression`**

Replace the `match expr` body in `src/parser/infer.rs` with:

```rust
match expr {
    HExpression::Int(_) => Some(Type::Int),
    HExpression::Bool(_) => Some(Type::Bool),
    HExpression::Str(_) => Some(Type::Str),
    HExpression::StructInitialization { name, fields: _ } => {
        Some(Type::Struct(name.to_string()))
    }
    HExpression::FunctionCall { fn_name, args: _ } => infer_function_call(fn_name, table),
    HExpression::Array {
        capacity: _,
        elements,
    } => infer_array(elements, table),
    HExpression::BinOp { lhs, op, rhs } => infer_binop(lhs, op, rhs, table),
    HExpression::Variable(_)
    | HExpression::Selff
    | HExpression::ArrayAccess { .. }
    | HExpression::FieldAccess { .. } => None,
}
```

Then add the helper above (keep alongside `infer_array`):

```rust
fn infer_binop(
    lhs: &HExpression,
    op: &crate::ast::hast::HBinOp,
    rhs: &HExpression,
    table: &SymbolTable,
) -> Option<Type> {
    use crate::ast::hast::HBinOp::*;
    match op {
        // Arithmetic and *Assign mirror the lhs/rhs types — currently always Int
        Addition | Subtraction | Multiplication | Division | Modulus
        | AddAssign | SubtractAssign | MultiplyAssign | DivideAssign => {
            // String concatenation: `+` where either operand is a string
            if matches!(op, Addition) {
                let lhs_ty = infer_expression(lhs, table);
                let rhs_ty = infer_expression(rhs, table);
                if matches!(lhs_ty, Some(Type::Str)) || matches!(rhs_ty, Some(Type::Str)) {
                    return Some(Type::Str);
                }
            }
            Some(Type::Int)
        }
        LessThan | LessThanOrEqual | GreaterThan | GreaterThanOrEqual | Equal | NotEqual => {
            Some(Type::Bool)
        }
        And | Or => Some(Type::Bool),
    }
}
```

- [ ] **Step 4: Run unit tests — verify all 5 pass**

Run: `cargo test --lib parser::infer::inference_tests`
Expected: PASS (5/5).

- [ ] **Step 5: Sanity-check `leapyear` now compiles to SSA**

Run: `cargo run -- --target qbe build examples/leapyear.sb -o /tmp/ly.ssa && tail -5 /tmp/ly.ssa`
Expected: emits SSA without "Missing type for variable" errors. (Whether it runs correctly is checked by integration test in later phase.)

- [ ] **Step 6: Commit**

```bash
git add src/parser/infer.rs
git commit -m "feat(parser): infer types of binary expressions (arith, cmp, logical, str-concat)"
```

---

## Phase 3 — Codegen Correctness

Each task in this phase fixes one or more example programs. After this phase, `ackermann`, `fib`, `greeter`, and `leapyear` should all be passing the integration test.

### Task 7: Fix "returns in all paths" check (unblocks `ackermann`)

**Files:**
- Modify: `src/generator/qbe.rs:170-229` (`generate_function`) and add a helper.

- [ ] **Step 1: Write a failing unit test reproducing the ackermann case**

Add to `src/generator/tests/qbe_tests.rs`:

```rust
#[test]
fn test_function_with_if_else_chain_all_paths_return() {
    // Reproduces ackermann.sb's structure:
    //   if c1 { return e1 } else if c2 { return e2 } else { return e3 }
    let inner_else = create_if_stmt(
        create_var_expr("c2"),
        create_return_stmt(Some(create_int_expr(2))),
        Some(create_return_stmt(Some(create_int_expr(3)))),
    );
    let outer = create_if_stmt(
        create_var_expr("c1"),
        create_return_stmt(Some(create_int_expr(1))),
        Some(inner_else),
    );
    let body = create_block_stmt(vec![
        create_declare_stmt("c1", AstType::Int, Some(create_int_expr(1))),
        create_declare_stmt("c2", AstType::Int, Some(create_int_expr(0))),
        outer,
    ]);
    let func = create_function("ack_like", Some(AstType::Int), body);
    let module = create_module(vec![func], Vec::new());

    // Should succeed (no "does not return in all code paths" error)
    let result = QbeGenerator::generate(module);
    assert!(
        result.is_ok(),
        "expected generation to succeed, got: {:?}",
        result
    );
}
```

- [ ] **Step 2: Run it — verify it fails**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_function_with_if_else_chain_all_paths_return`
Expected: FAIL with `Function 'ack_like' does not return in all code paths`.

- [ ] **Step 3: Replace the heuristic with a structural check on the AST**

In `src/generator/qbe.rs`, **add** this helper near the top of `impl QbeGenerator`:

```rust
/// Returns true if every path through `stmt` ends in a `return` (or `break`/`continue`,
/// which divert control out of the current function body's straight-line flow).
/// Pure AST analysis — does not depend on QBE block layout.
fn all_paths_return(stmt: &Statement) -> bool {
    match stmt {
        Statement::Return(_) => true,
        Statement::Block { statements, .. } => {
            statements.iter().any(QbeGenerator::all_paths_return)
        }
        Statement::If { body, else_branch, .. } => match else_branch {
            Some(else_b) => {
                QbeGenerator::all_paths_return(body)
                    && QbeGenerator::all_paths_return(else_b)
            }
            None => false,
        },
        // While/For may never run, so we can't claim they always return.
        _ => false,
    }
}
```

Then in `generate_function`, replace the `returns` block (currently using `qfunc.blocks.last()`) with:

```rust
let returns = QbeGenerator::all_paths_return(&func.body);

if !returns {
    if is_main {
        qfunc.add_instr(qbe::Instr::Ret(Some(qbe::Value::Const(0))));
    } else if func.ret_type.is_none() {
        qfunc.add_instr(qbe::Instr::Ret(None));
    } else {
        return Err(format!(
            "Function '{}' does not return in all code paths",
            &func.name
        ));
    }
}
```

- [ ] **Step 4: Run unit test — verify pass**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_function_with_if_else_chain_all_paths_return`
Expected: PASS.

- [ ] **Step 5: Run the ackermann integration test**

```bash
cargo test qbe_examples_ackermann -- --nocapture
```
Expected: PASS — stdout `61\n`. (`ackermann(3, 3) == 61`.)

If it instead fails because `println(int)` segfaults, that's expected and is fixed in Task 10. Add `#[ignore = "needs Task 10"]` above the test temporarily and continue.

- [ ] **Step 6: Commit**

```bash
git add src/generator/qbe.rs src/generator/tests/qbe_tests.rs
git commit -m "fix(qbe): detect 'returns on all paths' via AST analysis, not block tail"
```

### Task 8: Track function signatures and emit correct call return type

**Files:**
- Modify: `src/generator/qbe.rs` (`QbeGenerator` struct, `generate`, `generate_expression`'s `FunctionCall` arm)

- [ ] **Step 1: Write failing unit test**

Add to `src/generator/tests/qbe_tests.rs`:

```rust
#[test]
fn test_function_call_uses_callee_return_type() {
    // fn ptr_returner(): string { return "x" }
    // fn caller() { let p = ptr_returner() }
    let returner = create_function(
        "ptr_returner",
        Some(AstType::Str),
        create_return_stmt(Some(create_str_expr("x"))),
    );
    let caller = create_function(
        "caller",
        None,
        create_block_stmt(vec![create_declare_stmt(
            "p",
            AstType::Str,
            Some(create_call_expr("ptr_returner", vec![])),
        )]),
    );
    let module = create_module(vec![returner, caller], Vec::new());
    let result = QbeGenerator::generate(module).unwrap();
    let result_norm = normalize_qbe(&result);
    // Call must yield a long (pointer), not a word
    assert!(
        result_norm.contains("=l call $ptr_returner("),
        "expected `=l call`, got:\n{}",
        result_norm
    );
}
```

- [ ] **Step 2: Run it — verify it fails**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_function_call_uses_callee_return_type`
Expected: FAIL — current output has `=w call $ptr_returner(`.

- [ ] **Step 3: Add a function-signature table**

In `src/generator/qbe.rs`, modify the `QbeGenerator` struct:

```rust
pub struct QbeGenerator {
    tmp_counter: u32,
    scopes: Vec<HashMap<String, (qbe::Type<'static>, qbe::Value)>>,
    struct_map: HashMap<String, (qbe::Type<'static>, StructMeta, u64)>,
    /// Function name -> Some(return_type) for non-void, None for void
    func_returns: HashMap<String, Option<qbe::Type<'static>>>,
    loop_labels: Vec<String>,
    datadefs: Vec<qbe::DataDef<'static>>,
    typedefs: Vec<RcTypeDef>,
    module: qbe::Module<'static>,
}
```

In `generate` (after `tmp_counter: 0,` etc.):
```rust
func_returns: HashMap::new(),
```

After the structs loop and **before** the functions loop, do a pre-pass to populate the table:

```rust
for func in &prog.func {
    let ret_ty = match &func.ret_type {
        Some(ty) => Some(generator.get_type(ty.to_owned())?.into_abi()),
        None => None,
    };
    generator.func_returns.insert(func.name.clone(), ret_ty);
}
```

Also seed it with the runtime builtins so user code calling them gets the right types:

```rust
generator.func_returns.insert("_printf".into(), None);
generator.func_returns.insert("_exit".into(), None);
generator.func_returns.insert("_strcat".into(), Some(qbe::Type::Long));   // see Task 11
generator.func_returns.insert("_int_to_str".into(), Some(qbe::Type::Long)); // see Task 12
```

- [ ] **Step 4: Use the table in the `FunctionCall` arm**

In `generate_expression`, replace the `FunctionCall` match arm with:

```rust
Expression::FunctionCall { fn_name, args } => {
    let mut arg_results = Vec::new();
    for arg in args.iter() {
        arg_results.push(self.generate_expression(func, arg)?);
    }

    let ret_ty = self
        .func_returns
        .get(fn_name)
        .cloned()
        .unwrap_or(Some(qbe::Type::Word));

    match ret_ty {
        Some(ty) => {
            let tmp = self.new_temporary();
            func.assign_instr(
                tmp.clone(),
                ty.clone(),
                qbe::Instr::Call(fn_name.clone(), arg_results, None),
            );
            Ok((ty, tmp))
        }
        None => {
            // Void call: emit as a statement, return a word zero placeholder
            // (callers that ignore the result won't read it).
            func.add_instr(qbe::Instr::Call(fn_name.clone(), arg_results, None));
            Ok((qbe::Type::Word, qbe::Value::Const(0)))
        }
    }
}
```

- [ ] **Step 5: Run unit test — verify pass**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_function_call_uses_callee_return_type`
Expected: PASS.

- [ ] **Step 6: Run all unit tests — confirm no regression**

Run: `cargo test --lib`
Expected: all pre-existing tests still pass.

- [ ] **Step 7: Commit**

```bash
git add src/generator/qbe.rs src/generator/tests/qbe_tests.rs
git commit -m "feat(qbe): track function return types and emit correct call ABI"
```

### Task 9: Pick correct QBE arithmetic/comparison type from operands

**Files:**
- Modify: `src/generator/qbe.rs` (`generate_binop`)

- [ ] **Step 1: Write failing unit test**

Add to `src/generator/tests/qbe_tests.rs`:

```rust
#[test]
fn test_pointer_comparison_uses_long() {
    // let s1: string = "a"
    // let s2: string = "b"
    // return s1 == s2
    let body = create_block_stmt(vec![
        create_declare_stmt("s1", AstType::Str, Some(create_str_expr("a"))),
        create_declare_stmt("s2", AstType::Str, Some(create_str_expr("b"))),
        create_return_stmt(Some(create_binop_expr(
            create_var_expr("s1"),
            BinOp::Equal,
            create_var_expr("s2"),
        ))),
    ]);
    let func = create_function("ptr_eq", Some(AstType::Bool), body);
    let module = create_module(vec![func], Vec::new());

    let result = QbeGenerator::generate(module).unwrap();
    let result_norm = normalize_qbe(&result);
    // The compare should be over `l` (long) operands, not `w`
    assert!(
        result_norm.contains("ceql ") || result_norm.contains("=w ceql "),
        "expected long comparison ('ceql'), got:\n{}",
        result_norm
    );
}
```

- [ ] **Step 2: Run it — verify it fails**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_pointer_comparison_uses_long`
Expected: FAIL — current output uses `ceqw`.

- [ ] **Step 3: Update `generate_binop`**

Replace the body of `generate_binop` in `src/generator/qbe.rs` with:

```rust
fn generate_binop(
    &mut self,
    func: &mut qbe::Function<'static>,
    lhs: &Expression,
    op: &BinOp,
    rhs: &Expression,
) -> GeneratorResult<(qbe::Type<'static>, qbe::Value)> {
    let (lhs_ty, lhs_val) = self.generate_expression(func, lhs)?;
    let (rhs_ty, rhs_val) = self.generate_expression(func, rhs)?;
    let tmp = self.new_temporary();

    // Operate at the wider of the two operand types
    let operand_ty = Self::wider(&lhs_ty, &rhs_ty);

    let result_ty = match op {
        BinOp::LessThan
        | BinOp::LessThanOrEqual
        | BinOp::GreaterThan
        | BinOp::GreaterThanOrEqual
        | BinOp::Equal
        | BinOp::NotEqual => qbe::Type::Word, // bool/int comparison result
        _ => operand_ty.clone(),
    };

    func.assign_instr(
        tmp.clone(),
        result_ty.clone(),
        match op {
            BinOp::Addition | BinOp::AddAssign => qbe::Instr::Add(lhs_val.clone(), rhs_val.clone()),
            BinOp::Subtraction | BinOp::SubtractAssign => qbe::Instr::Sub(lhs_val.clone(), rhs_val.clone()),
            BinOp::Multiplication | BinOp::MultiplyAssign => qbe::Instr::Mul(lhs_val.clone(), rhs_val.clone()),
            BinOp::Division | BinOp::DivideAssign => qbe::Instr::Div(lhs_val.clone(), rhs_val.clone()),
            BinOp::Modulus => qbe::Instr::Rem(lhs_val.clone(), rhs_val.clone()),
            BinOp::And => qbe::Instr::And(lhs_val.clone(), rhs_val.clone()),
            BinOp::Or => qbe::Instr::Or(lhs_val.clone(), rhs_val.clone()),
            cmp => qbe::Instr::Cmp(
                operand_ty.clone(),
                match cmp {
                    BinOp::LessThan => qbe::Cmp::Slt,
                    BinOp::LessThanOrEqual => qbe::Cmp::Sle,
                    BinOp::GreaterThan => qbe::Cmp::Sgt,
                    BinOp::GreaterThanOrEqual => qbe::Cmp::Sge,
                    BinOp::Equal => qbe::Cmp::Eq,
                    BinOp::NotEqual => qbe::Cmp::Ne,
                    _ => unreachable!(),
                },
                lhs_val.clone(),
                rhs_val.clone(),
            ),
        },
    );

    match op {
        BinOp::AddAssign | BinOp::SubtractAssign | BinOp::MultiplyAssign | BinOp::DivideAssign => {
            self.generate_assignment(func, lhs, tmp.clone())?;
        }
        _ => {}
    };

    Ok((result_ty, tmp))
}

/// Returns whichever of `a`/`b` is the "wider" QBE numeric type.
/// Long > Word > {Halfword, Byte}; aggregates fall through as-is.
fn wider(a: &qbe::Type<'static>, b: &qbe::Type<'static>) -> qbe::Type<'static> {
    fn rank(t: &qbe::Type) -> u8 {
        match t {
            qbe::Type::Long | qbe::Type::Double => 4,
            qbe::Type::Word | qbe::Type::Single => 3,
            qbe::Type::Halfword
            | qbe::Type::SignedHalfword
            | qbe::Type::UnsignedHalfword => 2,
            qbe::Type::Byte
            | qbe::Type::SignedByte
            | qbe::Type::UnsignedByte => 1,
            _ => 0,
        }
    }
    if rank(a) >= rank(b) {
        a.clone()
    } else {
        b.clone()
    }
}
```

- [ ] **Step 4: Run unit test — verify pass**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_pointer_comparison_uses_long`
Expected: PASS.

- [ ] **Step 5: Run all unit tests — verify no regression**

Run: `cargo test --lib generator::tests::qbe_tests`
Expected: all green.

- [ ] **Step 6: Commit**

```bash
git add src/generator/qbe.rs src/generator/tests/qbe_tests.rs
git commit -m "fix(qbe): pick binop type from wider operand; emit correct cmp suffix"
```

### Task 10: Add `_int_to_str` runtime + auto-stringify ints in `println`

**Files:**
- Modify: `builtin/builtin.c`
- Modify: `lib/io.sb`
- Modify: `src/generator/qbe.rs` (call site lowering for `print`/`println` with non-string arg)

**Design choice:** Rather than overloading dispatch (the language has no overloading), we expose a stdlib helper `println_int(n: int)` that calls `_int_to_str` then `_printf`. The QBE backend rewrites `println(<int-typed expr>)` into a call to `println_int` at codegen time. This keeps the change local to QBE; JS target is unaffected.

- [ ] **Step 1: Add `_int_to_str` to `builtin/builtin.c`**

Append to `builtin/builtin.c` (before `/* END builtins */`):

```c
static char _int_to_str_buf[32];

char *_int_to_str(int n)
{
    snprintf(_int_to_str_buf, sizeof(_int_to_str_buf), "%d", n);
    return _int_to_str_buf;
}
```

- [ ] **Step 2: Expose it in the Antimony stdlib**

Append to `lib/io.sb`:

```rust
// Implemented by each backend (returns a heap-managed string for an int)
fn _int_to_str(n: int): string

// Print an integer followed by newline.
fn println_int(n: int) {
    println(_int_to_str(n))
}

// Print an integer (no newline).
fn print_int(n: int) {
    print(_int_to_str(n))
}
```

- [ ] **Step 3: Lower `println(int)` / `print(int)` in QBE backend**

In `src/generator/qbe.rs`, modify the `Expression::FunctionCall` arm of `generate_expression` to add a tiny rewrite *before* doing argument codegen:

```rust
Expression::FunctionCall { fn_name, args } => {
    // Rewrite println(<int>) -> println_int(<int>) when the lone arg is int-typed.
    // Same for print().
    let mut effective_name = fn_name.clone();
    if (fn_name == "println" || fn_name == "print") && args.len() == 1 {
        // Probe the arg type with a sub-codegen on a scratch fn isn't worth it;
        // do a cheap structural guess: literals + arithmetic-shaped exprs are int.
        if Self::expression_is_int_typed(&args[0], &self.func_returns) {
            effective_name = format!("{}_int", fn_name);
        }
    }

    let mut arg_results = Vec::new();
    for arg in args.iter() {
        arg_results.push(self.generate_expression(func, arg)?);
    }

    let ret_ty = self
        .func_returns
        .get(&effective_name)
        .cloned()
        .unwrap_or(Some(qbe::Type::Word));

    match ret_ty {
        Some(ty) => {
            let tmp = self.new_temporary();
            func.assign_instr(
                tmp.clone(),
                ty.clone(),
                qbe::Instr::Call(effective_name, arg_results, None),
            );
            Ok((ty, tmp))
        }
        None => {
            func.add_instr(qbe::Instr::Call(effective_name, arg_results, None));
            Ok((qbe::Type::Word, qbe::Value::Const(0)))
        }
    }
}
```

Add the helper near `wider`:

```rust
fn expression_is_int_typed(
    expr: &Expression,
    func_returns: &HashMap<String, Option<qbe::Type<'static>>>,
) -> bool {
    use crate::ast::BinOp::*;
    match expr {
        Expression::Int(_) => true,
        Expression::BinOp { op, .. } => matches!(
            op,
            Addition | Subtraction | Multiplication | Division | Modulus
        ),
        Expression::FunctionCall { fn_name, .. } => matches!(
            func_returns.get(fn_name),
            Some(Some(qbe::Type::Word))
        ) && fn_name != "println" && fn_name != "print",
        _ => false,
    }
}
```

- [ ] **Step 4: Run the `fib` integration test**

```bash
cargo test qbe_examples_fib -- --nocapture
```
Expected: PASS — stdout `55\n`.

- [ ] **Step 5: Run the `ackermann` integration test (remove the `#[ignore]` from Task 7 if added)**

```bash
cargo test qbe_examples_ackermann -- --nocapture
```
Expected: PASS — stdout `61\n`.

- [ ] **Step 6: Commit**

```bash
git add builtin/builtin.c lib/io.sb src/generator/qbe.rs
git commit -m "feat(qbe): lower println(int)/print(int) to a stdlib int->str helper"
```

### Task 11: Lower string concatenation to `_strcat` builtin (unblocks `greeter`)

**Files:**
- Modify: `builtin/builtin.c`
- Modify: `lib/io.sb`
- Modify: `src/generator/qbe.rs` (`generate_binop`)

- [ ] **Step 1: Add `_strcat` to `builtin/builtin.c`**

Append (before `/* END builtins */`):
```c
#include <string.h>

char *_strcat(char *a, char *b)
{
    size_t la = strlen(a);
    size_t lb = strlen(b);
    char *out = (char *)malloc(la + lb + 1);
    memcpy(out, a, la);
    memcpy(out + la, b, lb);
    out[la + lb] = '\0';
    return out;
}
```

- [ ] **Step 2: Expose it in `lib/io.sb`**

Append:
```rust
// Concatenate two strings into a freshly allocated string.
fn _strcat(a: string, b: string): string
```

(Native declaration; no body. Backend resolves to the C runtime symbol.)

- [ ] **Step 3: In `generate_binop`, intercept `Addition` over `Long` operands**

After `let operand_ty = Self::wider(&lhs_ty, &rhs_ty);` add:

```rust
// String concatenation: '+' on pointer-typed operands is a runtime call.
if matches!(op, BinOp::Addition)
    && matches!(operand_ty, qbe::Type::Long)
    && matches!(lhs_ty, qbe::Type::Long)
    && matches!(rhs_ty, qbe::Type::Long)
{
    let tmp = self.new_temporary();
    func.assign_instr(
        tmp.clone(),
        qbe::Type::Long,
        qbe::Instr::Call(
            "_strcat".to_owned(),
            vec![
                (qbe::Type::Long, lhs_val.clone()),
                (qbe::Type::Long, rhs_val.clone()),
            ],
            None,
        ),
    );
    return Ok((qbe::Type::Long, tmp));
}
```

- [ ] **Step 4: Run greeter integration test**

```bash
cargo test qbe_examples_greeter -- --nocapture
```
Expected: PASS — stdout `Hello World\n`.

- [ ] **Step 5: Commit**

```bash
git add builtin/builtin.c lib/io.sb src/generator/qbe.rs
git commit -m "feat(qbe): lower string concatenation to _strcat runtime helper"
```

### Task 12: Inline `fn x() = expr` should infer return type (also unblocks `greeter` cleanly)

**Files:**
- Modify: `src/parser/infer.rs`

**Background:** `fn greet(name: string) = "Hello " + name` is parsed with `ret_type: None`, body `Return("Hello "+name)`. When the function-body inference treats the return expression, it should backfill `ret_type`.

- [ ] **Step 1: Write failing unit test**

Add to `inference_tests` in `src/parser/infer.rs`:

```rust
#[test]
fn infers_function_return_type_from_inline_body() {
    use crate::ast::hast::{HFunction, HModule, HStatement, HVariable};
    let mut m = HModule {
        imports: HashSet::new(),
        func: vec![HFunction {
            name: "greet".into(),
            arguments: vec![HVariable {
                name: "name".into(),
                ty: Some(Type::Str),
            }],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Return(Some(HExpression::BinOp {
                    lhs: Box::new(HExpression::Str("Hello ".into())),
                    op: crate::ast::hast::HBinOp::Addition,
                    rhs: Box::new(HExpression::Variable("name".into())),
                }))],
                scope: vec![],
            },
        }],
        structs: vec![],
        globals: vec![],
    };
    infer(&mut m);
    assert_eq!(m.func[0].ret_type, Some(Type::Str));
}
```

- [ ] **Step 2: Run test — verify it fails**

Run: `cargo test --lib parser::infer::inference_tests::infers_function_return_type_from_inline_body`
Expected: FAIL.

- [ ] **Step 3: Extend the `infer` function in `src/parser/infer.rs`**

In the `for func in &mut program.func` loop, **before** iterating statements, add:

```rust
// Backfill missing return type from a single trailing `return <expr>`
if func.ret_type.is_none() {
    if let HStatement::Block { statements, .. } = &func.body {
        if let Some(HStatement::Return(Some(expr))) = statements.last() {
            func.ret_type = infer_expression(expr, table);
        }
    }
}
```

- [ ] **Step 4: Verify the test passes**

Run: `cargo test --lib parser::infer::inference_tests::infers_function_return_type_from_inline_body`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/parser/infer.rs
git commit -m "feat(parser): infer return type for inline 'fn name() = expr' bodies"
```

### Task 13: Run leapyear integration test

**Files:** none (this task only verifies prior fixes)

- [ ] **Step 1: Run integration test**

```bash
cargo test qbe_examples_leapyear -- --nocapture
```

Expected: PASS — stdout `Leap year\n` (2020 is divisible by 4 and not by 100, so the source's `divisibleBy4 && divisibleBy100` is true).

If it fails, investigate before proceeding. Likely root causes:
- Bool stored as Byte but binop result is Word — adjust `generate_assignment` to insert a width conversion when storing into a `Byte` slot.
- Comparison result wrong sign in `generate_binop`.

- [ ] **Step 2: If it passed without further code changes, commit a marker test only**

```bash
git commit --allow-empty -m "test(qbe): leapyear integration test now passes (no code change needed)"
```

---

## Phase 4 — Structs, Arrays, Loops

After this phase, `sandbox`, `loops`, and `bubblesort` should pass.

### Task 14: Look up structs by AST name, not placeholder QBE type (unblocks `sandbox`)

**Files:**
- Modify: `src/generator/qbe.rs` — change `struct_map` indexing semantics and tag every struct-typed `qbe::Value` with a name.

**Strategy:** Replace the QBE `Type::Word` placeholder used as the struct key with the actual aggregate `qbe::Type::Aggregate(...)`. Currently the struct lookup matches `if ty == sty` against placeholders; we change that to track each struct variable's *AST struct name* in the scope alongside its qbe::Value.

- [ ] **Step 1: Write failing unit test**

Add to `src/generator/tests/qbe_tests.rs`:

```rust
#[test]
fn test_nested_struct_field_access_resolves_correct_struct() {
    // struct Point { x: int, y: int }
    // struct Rectangle { origin: Point, w: int }
    // fn main() { let r = Rectangle { ... }; r.origin.x }
    let point = create_struct_def("Point", vec![
        create_variable("x", AstType::Int),
        create_variable("y", AstType::Int),
    ]);
    let rect = create_struct_def("Rectangle", vec![
        Variable { name: "origin".into(), ty: Some(AstType::Struct("Point".into())) },
        create_variable("w", AstType::Int),
    ]);

    let init = Expression::StructInitialization {
        name: "Rectangle".into(),
        fields: {
            let mut m = std::collections::HashMap::new();
            m.insert("origin".into(), Box::new(Expression::StructInitialization {
                name: "Point".into(),
                fields: {
                    let mut p = std::collections::HashMap::new();
                    p.insert("x".into(), Box::new(create_int_expr(10)));
                    p.insert("y".into(), Box::new(create_int_expr(20)));
                    p
                },
            }));
            m.insert("w".into(), Box::new(create_int_expr(99)));
            m
        },
    };

    let body = create_block_stmt(vec![
        create_declare_stmt("r", AstType::Struct("Rectangle".into()), Some(init)),
        Statement::Exp(Expression::FieldAccess {
            expr: Box::new(Expression::FieldAccess {
                expr: Box::new(create_var_expr("r")),
                field: Box::new(create_var_expr("origin")),
            }),
            field: Box::new(create_var_expr("x")),
        }),
    ]);
    let func = create_function("main", None, body);
    let module = create_module(vec![func], vec![point, rect]);

    // Should compile without "No field 'origin' on struct Point"
    let result = QbeGenerator::generate(module);
    assert!(result.is_ok(), "got: {:?}", result);
}
```

- [ ] **Step 2: Run it — verify it fails**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_nested_struct_field_access_resolves_correct_struct`
Expected: FAIL with `No field 'origin' on struct Point`.

- [ ] **Step 3: Refactor scopes to carry the struct's AST name when applicable**

Change scope value type:

```rust
struct VarBinding {
    qty: qbe::Type<'static>,
    /// AST struct name when this binding refers to a struct value.
    struct_name: Option<String>,
    value: qbe::Value,
}

scopes: Vec<HashMap<String, VarBinding>>,
```

Update `new_var` to take an optional struct name:

```rust
fn new_var(
    &mut self,
    ty: &qbe::Type<'static>,
    struct_name: Option<&str>,
    name: &str,
) -> GeneratorResult<qbe::Value> {
    if self.get_var(name).is_ok() {
        return Err(format!("Re-declaration of variable '{}'", name));
    }
    let tmp = self.new_temporary();
    let scope = self.scopes.last_mut().expect("expected last scope to be present");
    scope.insert(
        name.to_owned(),
        VarBinding {
            qty: ty.to_owned(),
            struct_name: struct_name.map(|s| s.to_owned()),
            value: tmp.to_owned(),
        },
    );
    Ok(tmp)
}
```

Update `get_var` to return `&VarBinding`. All call sites must be updated.

- [ ] **Step 4: Pass struct names through declarations**

In `generate_statement`'s `Declare` arm, when the declared type is `Type::Struct(name)`, pass the name to `new_var`:

```rust
Statement::Declare { variable, value } => {
    let raw_ty = variable.ty.as_ref().ok_or_else(|| format!("Missing type for variable '{}'", &variable.name))?.to_owned();
    let struct_name = match &raw_ty {
        Type::Struct(n) => Some(n.as_str()),
        _ => None,
    };
    let ty = self.get_type(raw_ty.clone())?;
    let tmp = self.new_var(&ty, struct_name, &variable.name)?;
    if let Some(expr) = value {
        let (expr_type, expr_value) = self.generate_expression(func, expr)?;
        func.assign_instr(tmp, expr_type, qbe::Instr::Copy(expr_value));
    }
}
```

Same for function arguments in `generate_function`:
```rust
let raw_ty = arg.ty.as_ref().ok_or("Function arguments must have a type")?.to_owned();
let struct_name = match &raw_ty { Type::Struct(n) => Some(n.as_str()), _ => None };
let ty = self.get_type(raw_ty)?;
let tmp = self.new_var(&ty, struct_name, &arg.name)?;
```

- [ ] **Step 5: Make `resolve_field_access` use the binding's struct name**

Replace the body of `resolve_field_access`:

```rust
fn resolve_field_access(
    &mut self,
    obj: &Expression,
    field: &Expression,
) -> GeneratorResult<(qbe::Value, qbe::Type<'static>, u64)> {
    let (src, struct_name, off) = match obj {
        Expression::Variable(var) => {
            let binding = self.get_var(var)?;
            let name = binding
                .struct_name
                .clone()
                .ok_or_else(|| format!("Variable '{}' is not a struct", var))?;
            (binding.value.clone(), name, 0)
        }
        Expression::FieldAccess { expr, field } => {
            // Recurse to get parent (src, parent_struct_name, off_so_far)
            let (src, parent_ty, off) = self.resolve_field_access(expr, field)?;
            // The recursive call returned the *field*'s type/offset. We need
            // the struct name of the field. Look it up by qbe::Type identity:
            // since we now register structs with their real Aggregate type,
            // we can map back. See helper struct_name_for_type below.
            let name = self
                .struct_name_for_type(&parent_ty)
                .ok_or_else(|| format!("Field is not a struct"))?;
            (src, name, off)
        }
        Expression::Selff => return Err("methods not implemented".to_owned()),
        other => {
            return Err(format!(
                "Invalid field access type: expected variable, field access or 'self', got {:?}",
                other,
            ));
        }
    };
    let field_name = match field {
        Expression::Variable(v) => v.clone(),
        Expression::FunctionCall { .. } => return Err("methods not implemented".to_owned()),
        _ => unreachable!(),
    };

    let meta = &self
        .struct_map
        .get(&struct_name)
        .ok_or_else(|| format!("Unknown struct '{}'", struct_name))?
        .1;

    let (field_ty, field_offset) = meta
        .get(&field_name)
        .ok_or_else(|| format!("No field '{}' on struct {}", field_name, struct_name))?
        .to_owned();

    Ok((src, field_ty, field_offset + off))
}

/// Look up the AST struct name for a given QBE type (Aggregate-keyed).
fn struct_name_for_type(&self, ty: &qbe::Type<'static>) -> Option<String> {
    self.struct_map
        .iter()
        .find_map(|(n, (sty, _, _))| if sty == ty { Some(n.clone()) } else { None })
}
```

- [ ] **Step 6: Register structs with their actual aggregate type**

In `generate`, after `module.add_type((*typedef_rc).clone());`, store the aggregate type in `struct_map` instead of the placeholder:

```rust
let aggregate_ty = unsafe {
    std::mem::transmute::<qbe::Type<'_>, qbe::Type<'static>>(
        qbe::Type::Aggregate(generator.typedefs.last().unwrap()),
    )
};

// Replace placeholder Word with real aggregate
if let Some(entry) = generator.struct_map.get_mut(&def.name) {
    entry.0 = aggregate_ty;
}
```

(`generate_struct` still inserts with placeholder Word; this fixes it up after the `Rc<TypeDef>` is in the vec.)

- [ ] **Step 7: Update `get_type` to return aggregate for structs**

```rust
Type::Struct(name) => {
    let (ty, ..) = self
        .struct_map
        .get(&name)
        .ok_or_else(|| format!("Use of undeclared struct '{}'", name))?
        .to_owned();
    Ok(ty)
}
```

(Same code as today; what changes is that `ty` is now the real aggregate, not Word.)

- [ ] **Step 8: Run unit test — verify pass**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_nested_struct_field_access_resolves_correct_struct`
Expected: PASS.

- [ ] **Step 9: Run sandbox integration test**

```bash
cargo test qbe_examples_sandbox -- --nocapture
```
Expected: PASS (stdout empty, exit 0 — `sandbox.sb`'s `main` returns 0).

- [ ] **Step 10: Commit**

```bash
git add src/generator/qbe.rs src/generator/tests/qbe_tests.rs
git commit -m "fix(qbe): resolve struct field access by AST name, not placeholder type"
```

### Task 15: Implement `Expression::ArrayAccess` as rvalue

**Files:**
- Modify: `src/generator/qbe.rs`

- [ ] **Step 1: Failing test**

Add to `qbe_tests.rs`:

```rust
#[test]
fn test_array_access_rvalue_emits_load() {
    // let a = [1, 2, 3]; return a[1]
    let arr = Expression::Array {
        capacity: 3,
        elements: vec![create_int_expr(1), create_int_expr(2), create_int_expr(3)],
    };
    let body = create_block_stmt(vec![
        create_declare_stmt(
            "a",
            AstType::Array(Box::new(AstType::Int), Some(3)),
            Some(arr),
        ),
        create_return_stmt(Some(Expression::ArrayAccess {
            name: "a".into(),
            index: Box::new(create_int_expr(1)),
        })),
    ]);
    let func = create_function("get", Some(AstType::Int), body);
    let module = create_module(vec![func], Vec::new());
    let result = QbeGenerator::generate(module).unwrap();
    let result_norm = normalize_qbe(&result);
    assert!(result_norm.contains("loadw "), "expected loadw, got:\n{}", result_norm);
}
```

- [ ] **Step 2: Implement the arm**

In `generate_expression`, add (before the `_ => todo!(...)` fallback):

```rust
Expression::ArrayAccess { name, index } => {
    let binding = self.get_var(name)?.clone();
    // Array memory layout (see generate_array): { length: l, elem0: T, elem1: T, ... }
    let elem_ty = qbe::Type::Word; // TODO: track per-array element type alongside binding
    let elem_size = self.type_size(&elem_ty);

    let (_, idx_val) = self.generate_expression(func, index)?;

    let scaled = self.new_temporary();
    func.assign_instr(
        scaled.clone(),
        qbe::Type::Long,
        qbe::Instr::Mul(idx_val, qbe::Value::Const(elem_size)),
    );
    let with_header = self.new_temporary();
    func.assign_instr(
        with_header.clone(),
        qbe::Type::Long,
        qbe::Instr::Add(scaled, qbe::Value::Const(8)),
    );
    let addr = self.new_temporary();
    func.assign_instr(
        addr.clone(),
        qbe::Type::Long,
        qbe::Instr::Add(binding.value.clone(), with_header),
    );

    let result = self.new_temporary();
    func.assign_instr(
        result.clone(),
        elem_ty.clone(),
        qbe::Instr::Load(elem_ty.clone(), addr),
    );
    Ok((elem_ty, result))
}
```

(`elem_ty` is hardcoded to Word for this iteration — sufficient for `bubblesort.sb` (int array) and `loops.sb` (string array, which we treat below). A follow-up plan will track per-array element types in the binding.)

For string arrays (used in `loops.sb`), generalize: when the binding's qty is `Long` (a pointer), we still want Word loads — wait, that's wrong. **Refinement:** track element type in the binding by extending `VarBinding`:

```rust
struct VarBinding {
    qty: qbe::Type<'static>,
    struct_name: Option<String>,
    /// Element type when this binding is an array.
    array_elem_ty: Option<qbe::Type<'static>>,
    value: qbe::Value,
}
```

In `Statement::Declare`, when `Type::Array(inner, _)`, compute `array_elem_ty`:
```rust
let elem_ty = match &raw_ty {
    Type::Array(inner, _) => Some(self.get_type(*inner.clone())?),
    _ => None,
};
```
Pass to `new_var` (extend signature similarly).

Then in `Expression::ArrayAccess`, prefer `binding.array_elem_ty.clone().unwrap_or(qbe::Type::Word)`.

- [ ] **Step 3: Run unit test — verify pass**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_array_access_rvalue_emits_load`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/generator/qbe.rs src/generator/tests/qbe_tests.rs
git commit -m "feat(qbe): implement array indexing as rvalue with per-array elem type"
```

### Task 16: Implement `arr[i] = v` (ArrayAccess as lvalue)

**Files:**
- Modify: `src/generator/qbe.rs` — `generate_assignment`

- [ ] **Step 1: Failing test**

```rust
#[test]
fn test_array_access_lvalue_emits_store() {
    // let a = [0, 0, 0]; a[1] = 42; return a[1]
    let arr = Expression::Array {
        capacity: 3,
        elements: vec![create_int_expr(0); 3],
    };
    let body = create_block_stmt(vec![
        create_declare_stmt("a", AstType::Array(Box::new(AstType::Int), Some(3)), Some(arr)),
        create_assign_stmt(
            Expression::ArrayAccess { name: "a".into(), index: Box::new(create_int_expr(1)) },
            create_int_expr(42),
        ),
        create_return_stmt(Some(Expression::ArrayAccess {
            name: "a".into(),
            index: Box::new(create_int_expr(1)),
        })),
    ]);
    let func = create_function("write", Some(AstType::Int), body);
    let module = create_module(vec![func], Vec::new());
    let result = QbeGenerator::generate(module).unwrap();
    let result_norm = normalize_qbe(&result);
    assert!(result_norm.contains("storew 42"), "expected storew 42, got:\n{}", result_norm);
}
```

- [ ] **Step 2: Replace `todo!()` arm in `generate_assignment`**

```rust
Expression::ArrayAccess { name, index } => {
    let binding = self.get_var(name)?.clone();
    let elem_ty = binding.array_elem_ty.clone().unwrap_or(qbe::Type::Word);
    let elem_size = self.type_size(&elem_ty);

    let (_, idx_val) = self.generate_expression(func, index)?;

    let scaled = self.new_temporary();
    func.assign_instr(scaled.clone(), qbe::Type::Long,
        qbe::Instr::Mul(idx_val, qbe::Value::Const(elem_size)));
    let with_header = self.new_temporary();
    func.assign_instr(with_header.clone(), qbe::Type::Long,
        qbe::Instr::Add(scaled, qbe::Value::Const(8)));
    let addr = self.new_temporary();
    func.assign_instr(addr.clone(), qbe::Type::Long,
        qbe::Instr::Add(binding.value, with_header));

    func.add_instr(qbe::Instr::Store(elem_ty, addr, rhs));
}
```

- [ ] **Step 3: Run test — verify pass**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_array_access_lvalue_emits_store`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/generator/qbe.rs src/generator/tests/qbe_tests.rs
git commit -m "feat(qbe): implement array indexing as lvalue with typed store"
```

### Task 17: Fix stdlib `len()` to use length header

**Files:**
- Modify: `lib/array.sb`

The QBE backend's `generate_array` (qbe.rs:782-798) stores `length` at offset 0 of the array. The current `len()` walks until it hits a falsy element — that's wrong even in JS land for arrays containing zero. We replace it with a direct read of slot 0.

But: Antimony has no syntax to read raw memory from inside a `.sb` file. The cheapest fix is to expose `_array_len` as a backend builtin and have `len` call it.

- [ ] **Step 1: Add `_array_len` to `builtin/builtin.c`**

```c
#include <stdint.h>

int64_t _array_len(int64_t *arr)
{
    return arr[0];
}
```

- [ ] **Step 2: Replace `lib/array.sb` `len()` body**

```rust
fn _array_len(arr: int[]): int

fn len(arr: int[]): int {
    return _array_len(arr)
}
```

(Yes, `int[]` doesn't perfectly match a pointer in the stdlib's type system, but the QBE backend treats arrays as Long pointers — so the call is ABI-correct.)

- [ ] **Step 3: Register `_array_len` in `func_returns`**

In `src/generator/qbe.rs::generate`, alongside the other builtin registrations:
```rust
generator.func_returns.insert("_array_len".into(), Some(qbe::Type::Long));
```

- [ ] **Step 4: Run bubblesort manually to confirm it gets past `len()`**

```bash
cargo run -- --target qbe build examples/bubblesort.sb -o /tmp/bs.ssa
qbe -o /tmp/bs.s /tmp/bs.ssa && cc /tmp/bs.s builtin/builtin.c -o /tmp/bs.exe && /tmp/bs.exe
```
Expected: this might still fail because `println(arr)` doesn't have a printer. Note this and continue to Task 18.

- [ ] **Step 5: Commit**

```bash
git add lib/array.sb builtin/builtin.c src/generator/qbe.rs
git commit -m "fix(stdlib): make len() read array length from header instead of scanning"
```

### Task 18: Implement `Statement::For` (unblocks `loops`)

**Files:**
- Modify: `src/generator/qbe.rs` — `generate_statement`, add `generate_for` helper.

- [ ] **Step 1: Failing test**

```rust
#[test]
fn test_for_loop_emits_while_lowering() {
    // for x in [1,2,3] { print_int(x) }
    let arr = Expression::Array {
        capacity: 3,
        elements: vec![create_int_expr(1), create_int_expr(2), create_int_expr(3)],
    };
    let body = create_block_stmt(vec![Statement::Exp(create_call_expr("print_int", vec![create_var_expr("x")]))]);
    let for_stmt = Statement::For {
        ident: Variable { name: "x".into(), ty: Some(AstType::Int) },
        expr: arr,
        body: Box::new(body),
    };
    let outer = create_block_stmt(vec![for_stmt]);
    let func = create_function("loop_test", None, outer);
    let module = create_module(vec![func], Vec::new());
    let result = QbeGenerator::generate(module).unwrap();
    let n = normalize_qbe(&result);
    assert!(n.contains("@loop"));
    assert!(n.contains("call $print_int"));
}
```

- [ ] **Step 2: Implement `generate_for`**

In `src/generator/qbe.rs::generate_statement`, replace the `_ => todo!(...)` fallback's `For` part by adding:

```rust
Statement::For { ident, expr, body } => {
    self.generate_for(func, ident, expr, body)?;
}
```

Add the helper:

```rust
fn generate_for(
    &mut self,
    func: &mut qbe::Function<'static>,
    ident: &Variable,
    expr: &Expression,
    body: &Statement,
) -> GeneratorResult<()> {
    self.scopes.push(HashMap::new());

    // 1. Eval the iterable once into a temp pointer
    let (arr_ty, arr_ptr) = self.generate_expression(func, expr)?;

    // 2. Read length from offset 0
    let len_tmp = self.new_temporary();
    func.assign_instr(
        len_tmp.clone(),
        qbe::Type::Long,
        qbe::Instr::Load(qbe::Type::Long, arr_ptr.clone()),
    );

    // 3. Allocate iteration index var (i64)
    let i_tmp = self.new_temporary();
    func.assign_instr(i_tmp.clone(), qbe::Type::Long, qbe::Instr::Copy(qbe::Value::Const(0)));

    // 4. Allocate the loop variable (gets re-assigned each iteration)
    let elem_ty = match &arr_ty {
        // We don't currently propagate element type out of generate_expression for arrays;
        // fall back to ident's declared type.
        _ => self.get_type(ident.ty.clone().ok_or("for: ident must have type")?)?,
    };
    let loop_var = self.new_var(&elem_ty, None, &ident.name)?;

    // 5. Emit cond/body/end blocks
    self.tmp_counter += 1;
    let label = format!("loop.{}", self.tmp_counter);
    let cond_label = format!("{}.cond", label);
    let body_label = format!("{}.body", label);
    let end_label = format!("{}.end", label);

    self.loop_labels.push(label);
    func.add_block(cond_label.clone());

    let in_range = self.new_temporary();
    func.assign_instr(
        in_range.clone(),
        qbe::Type::Word,
        qbe::Instr::Cmp(qbe::Type::Long, qbe::Cmp::Slt, i_tmp.clone(), len_tmp.clone()),
    );
    func.add_instr(qbe::Instr::Jnz(in_range, body_label.clone(), end_label.clone()));

    func.add_block(body_label);

    // Load arr[i] into loop_var
    let elem_size = self.type_size(&elem_ty);
    let scaled = self.new_temporary();
    func.assign_instr(scaled.clone(), qbe::Type::Long, qbe::Instr::Mul(i_tmp.clone(), qbe::Value::Const(elem_size)));
    let with_header = self.new_temporary();
    func.assign_instr(with_header.clone(), qbe::Type::Long, qbe::Instr::Add(scaled, qbe::Value::Const(8)));
    let addr = self.new_temporary();
    func.assign_instr(addr.clone(), qbe::Type::Long, qbe::Instr::Add(arr_ptr.clone(), with_header));
    func.assign_instr(loop_var.clone(), elem_ty.clone(), qbe::Instr::Load(elem_ty.clone(), addr));

    self.generate_statement(func, body)?;

    // i += 1
    let next_i = self.new_temporary();
    func.assign_instr(next_i.clone(), qbe::Type::Long, qbe::Instr::Add(i_tmp.clone(), qbe::Value::Const(1)));
    func.assign_instr(i_tmp.clone(), qbe::Type::Long, qbe::Instr::Copy(next_i));

    if !func.blocks.last().is_some_and(|b| b.jumps()) {
        func.add_instr(qbe::Instr::Jmp(cond_label));
    }

    func.add_block(end_label);

    self.loop_labels.pop();
    self.scopes.pop();
    Ok(())
}
```

- [ ] **Step 3: Run unit test — verify pass**

Run: `cargo test --lib generator::tests::qbe_tests::tests::test_for_loop_emits_while_lowering`
Expected: PASS.

- [ ] **Step 4: Run loops integration test**

```bash
cargo test qbe_examples_loops -- --nocapture
```
Expected: PASS — stdout matches `expected/loops.txt`.

- [ ] **Step 5: Commit**

```bash
git add src/generator/qbe.rs src/generator/tests/qbe_tests.rs
git commit -m "feat(qbe): lower 'for x in arr' to indexed while loop"
```

### Task 19: Add `println_arr` / array-of-int printer for `bubblesort.sb`

**Files:**
- Modify: `builtin/builtin.c`
- Modify: `lib/array.sb`
- Modify: `src/generator/qbe.rs`

`bubblesort.sb` ends with `println(arr)`. To match the JS reference (`[1, 2, 3, 4, 5]`) we add a runtime printer.

- [ ] **Step 1: Add `_println_int_arr` to `builtin/builtin.c`**

```c
void _println_int_arr(int64_t *arr)
{
    int64_t n = arr[0];
    int *cells = (int *)(arr + 1);
    printf("[");
    for (int64_t i = 0; i < n; ++i) {
        printf("%d", cells[i]);
        if (i + 1 < n) printf(", ");
    }
    printf("]\n");
}
```

- [ ] **Step 2: Declare and use it in stdlib**

`lib/array.sb` (append):
```rust
fn _println_int_arr(arr: int[])

fn println_int_arr(arr: int[]) {
    _println_int_arr(arr)
}
```

- [ ] **Step 3: Register return type and lower `println(<int[]>)`**

In `src/generator/qbe.rs::generate`:
```rust
generator.func_returns.insert("_println_int_arr".into(), None);
generator.func_returns.insert("println_int_arr".into(), None);
```

Extend the rewrite in `Expression::FunctionCall`:
```rust
if (fn_name == "println" || fn_name == "print") && args.len() == 1 {
    if Self::expression_is_int_typed(&args[0], &self.func_returns) {
        effective_name = format!("{}_int", fn_name);
    } else if Self::expression_is_int_array(&args[0]) {
        effective_name = format!("{}_int_arr", fn_name);
    }
}
```

Add helper:
```rust
fn expression_is_int_array(expr: &Expression) -> bool {
    matches!(expr, Expression::Array { .. })
        // For variables, we'd need scope lookup — simplification: a Variable
        // bound to an int[] is detected at use site below
}
```

For the `Variable("arr")` case in `bubblesort.sb`, extend by consulting `self.get_var(name)`'s `array_elem_ty`. Simpler to inline at the `FunctionCall` arm (move detection out of the static helper):

```rust
if (fn_name == "println" || fn_name == "print") && args.len() == 1 {
    if Self::expression_is_int_typed(&args[0], &self.func_returns) {
        effective_name = format!("{}_int", fn_name);
    } else if let Expression::Variable(n) = &args[0] {
        if let Ok(b) = self.get_var(n) {
            if matches!(b.array_elem_ty, Some(qbe::Type::Word)) {
                effective_name = format!("{}_int_arr", fn_name);
            }
        }
    } else if matches!(&args[0], Expression::Array { .. }) {
        effective_name = format!("{}_int_arr", fn_name);
    }
}
```

- [ ] **Step 4: Run bubblesort integration test**

```bash
cargo test qbe_examples_bubblesort -- --nocapture
```
Expected: PASS — stdout `[1, 2, 3, 4, 5]\n`.

- [ ] **Step 5: Commit**

```bash
git add builtin/builtin.c lib/array.sb src/generator/qbe.rs
git commit -m "feat(qbe): runtime printer for int arrays + dispatch from println"
```

---

## Phase 5 — Lock It In

### Task 20: All-green verification + README badge update

**Files:**
- Modify: `README.md` (status section)

- [ ] **Step 1: Run the full test suite**

```bash
cargo test
```
Expected: 100% green, including all 8 `qbe_examples_*` tests.

- [ ] **Step 2: Run each example one more time end-to-end**

```bash
for ex in examples/*.sb; do
    name=$(basename "$ex" .sb)
    echo "--- $name ---"
    cargo run --quiet -- --target qbe run "$ex"
    echo "exit=$?"
done
```
Expected: all 8 produce the golden stdout and exit 0.

- [ ] **Step 3: Update README**

Replace the line in `README.md` that reads:
```
The Antimony compiler emits JavaScript for the Node.js runtime, and a C backend is currently under development. Backends for WASM and LLVM are planned.
```

with:
```
The Antimony compiler emits JavaScript for the Node.js runtime. C and QBE backends are also supported (the QBE backend produces native code via QBE's SSA → assembly pipeline). Backends for WASM and LLVM are planned.
```

- [ ] **Step 4: Final commit**

```bash
git add README.md
git commit -m "docs(readme): mark QBE backend as supported"
```

---

## Future Work (Explicitly Out of Scope)

The following items from the original analysis are **deliberately deferred** to keep this plan focused on observable example correctness:

1. **Struct methods / `Selff`** — currently produce `unimplemented!`. No example uses methods.
2. **Top-level globals** — `prog.globals` is unused in the QBE generator. No example uses top-level globals.
3. **Replace `unsafe transmute` in `generate_array`** — sound under current architecture; cleaner solution requires `qbe` crate lifetime API changes.
4. **Separate counter for labels vs temps** — cosmetic; no correctness impact.
5. **Private linkage for module-internal symbols** — only matters once separate compilation lands.
6. **Remove debug `eprintln!` from struct generation** — not user-visible in `--release`. (Actually, do this opportunistically: gate behind a verbose flag.)
7. **`_memcpy` builtin** — relying on libc `memcpy` works because `cc` links libc by default.
8. **Per-element type tracking for arrays-of-strings, arrays-of-structs** — current implementation handles `int[]` and `string[]` rvalues for `for-each` adequately for the existing examples. A richer system (parametric `qbe::Type` per element) is future work.

A separate plan should pick these up after this plan is merged and stable.

---

## Self-Review

**Spec coverage** (against the issue list from the prior chat):

| Issue ID from analysis | Addressed by task |
|---|---|
| A1 (no stdlib for QBE) | Task 3 |
| A2 (no builtin lowering for QBE) | Task 5 (linkage), Task 1 (test harness uses cc + builtin.c) |
| A3 (println(int) segfault) | Task 10 |
| A4 (main exit code) | Task 4 |
| A5 (run_qbe gcc/CWD) | Task 5 |
| B6 (binop inference) | Task 6 |
| B7 (missing variable type → error) | Task 6 (most cases now infer) + Task 12 |
| C8 (returns-in-all-paths) | Task 7 |
| C9 (struct lookup by placeholder type) | Task 14 |
| C10 (inline fn return type) | Task 12 |
| C11 (binop result type Word) | Task 9 |
| C12 (FunctionCall always Word) | Task 8 |
| C13 (string concat as add) | Task 11 |
| C14 (Bool literal vs storage) | Acknowledged in self-critique; not addressed (only matters at struct/ABI boundaries; if `leapyear` regresses, fix in Task 13) |
| C15 (ArrayAccess) | Tasks 15, 16 |
| C16 (For loop) | Task 18 |
| C17 (methods / Selff) | Out of scope |
| C18 (struct ABI) | Out of scope |
| C19 (transmute) | Out of scope |
| C20 (counter split) | Out of scope |
| C21 (globals) | Out of scope |
| C22 (eprintln noise) | Out of scope (cosmetic) |
| C23 (memcpy builtin) | Out of scope (libc covers it) |
| C24 (linkage hygiene) | Out of scope |
| D25 (no QBE integration test) | Task 1 |
| D26 (qbe_tests too lax) | Implicitly addressed: each task adds tighter unit tests; further work in future plan |
| Bonus: builtin.c missing stdlib include | Task 2 |
| Bonus: stdlib `len()` broken | Task 17 |

**Placeholder scan:** No "TBD" / "implement later" — all code-changing steps include concrete code.

**Type consistency:**
- `func_returns: HashMap<String, Option<qbe::Type<'static>>>` — used identically across tasks 8, 10, 11, 17, 19.
- `VarBinding { qty, struct_name, array_elem_ty, value }` — fully shipped in Task 14, extended by one field in Task 15. All call sites in Tasks 15, 16, 18, 19 use this name.
- Method names: `wider`, `expression_is_int_typed`, `expression_is_int_array`, `all_paths_return`, `struct_name_for_type`, `generate_for` — all internal helpers, all newly defined in this plan, all referenced consistently.
- Builtin symbol names: `_printf`, `_exit`, `_strcat`, `_int_to_str`, `_array_len`, `_println_int_arr` — defined once each (Tasks 2, 11, 10, 17, 19) and registered identically.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-04-27-qbe-backend-hardening.md`. Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration. Recommended because phases are independent and each task has its own failing test as the success signal.
2. **Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach?
