# QBE Backend: Void-`main` Exit Code + Test Tightening

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the two remaining QBE-backend gaps that the survey on `origin/master` (`e26444d`) identified, and tighten the example-test runner so neither gap can recur silently.

**Architecture:** Three small, independent commits. (1) Make void source `main()` declare `function w $main()` at the QBE level and emit `ret 0` on every implicit-return path, so the OS exit code is deterministic. (2) Tighten `compile_and_run_qbe` in `src/tests/test_examples.rs` to assert exit-code 0 and either match against optional golden stdout or fail on empty stdout for `println`-using examples. (3) Either fix or explicitly mark `bubblesort.sb` so its existing silent wrong-codegen (`println(int[])` passes the array pointer to `_str_concat` as if it were a string) is caught and surfaced.

**Tech Stack:** Rust 1.x; `qbe` crate v4.0.0 (already on master); `qbe` CLI; `cc`. Existing master files: `src/generator/qbe.rs` (`generate_function`, ~ln 490–574), `src/tests/test_examples.rs` (`compile_and_run_qbe`, ~ln 165–229), `examples/bubblesort.sb`.

**Out of scope:** any broader QBE backend rewrite; methods/structs (already done upstream); array-to-string conversion (large surface — call out as a follow-up); `tests/qbe/*.sb` self-checking runner (that path already does the strict check).

---

## Pre-Flight: Survey Snapshot

Recorded at the start of this plan against `e26444d` (origin/master HEAD at 2026-04-27 12:15 UTC+2). All 9 examples in `examples/` go cleanly through `sb --target qbe build → qbe → cc → ./bin`. Run results:

| Example | stdout | exit | matches JS reference? |
|---|---|---|---|
| `args` | `Usage: args <arg1> <arg2> ...` | 0 | yes |
| `sandbox` | (empty) | 0 | yes (declared `fn main(): int { return 0 }`) |
| `hello_world` | `Hello World` | **12** | yes (stdout); no (exit) |
| `greeter` | `Hello World` | **12** | yes (stdout); no (exit) |
| `leapyear` | `Leap year` | **10** | yes (stdout); no (exit) |
| `fib` | `55` | **3** | yes (stdout); no (exit) |
| `ackermann` | `61` | **3** | yes (stdout); no (exit) |
| `loops` | (6 expected lines) | **1** | yes (stdout); no (exit) |
| `bubblesort` | (empty!) | 2 | **no** (JS prints `1,2,3,4,5`) |

**Two distinct issues:**

1. **Void `main()` exit code is `printf`'s last-call return value.** Bug: when source `main` is declared without `: int`, `generate_function` emits `function $main()` (no return type) followed by a bare `ret`. After the last `_printf` call, `w0` holds `printf`'s return (the byte count), and that's what the OS sees as the exit code (e.g., `Hello World\n` = 12 chars, `55\n` = 3 chars). Master's own example test runner explicitly works around this — see `src/tests/test_examples.rs` around line 215–224: *"Note: void main() may return a non-zero exit code in QBE since the backend doesn't yet emit `ret 0` for void functions"*. Affects 6/9 examples.

2. **`println(int[])` produces silent wrong codegen.** `bubblesort.sb` ends with `println(arr)` where `arr` is `int[]`. Master accepts this without typecheck error and emits `call $println(l <arr_ptr>)`. The `println` body does `_str_concat(<arr_ptr>, "\n")`, which calls `strlen()` on the array memory. Byte 0 of an int[] is the length cell's low byte (e.g., `5` for `[2,5,3,1,4]`); byte 1 is `0`, hitting the NUL terminator immediately. So `printf("%s")` prints one non-printable byte (`\x05`) — invisible — followed by `\n`. Net: empty observable stdout, exit 2 (1 invisible byte + 1 newline = `_printf`'s 2-byte return). The example "passes" master's QBE test only because the runner asserts `stdout doesn't contain "FAIL"` and an empty stdout doesn't.

3. **Meta-issue (test laxness):** `compile_and_run_qbe` (`src/tests/test_examples.rs:165–229`) only asserts that the binary wasn't killed by a signal — it accepts non-zero exits and doesn't compare stdout against any expected output. Both bugs above survive this check. The stricter `compile_and_run_qbe_checked` is only used for `tests/qbe/*.sb` self-checking files, not for the example pipeline. We tighten the example pipeline as part of Task 2 below.

---

## Working Conventions

- Branch: `cursor/c7723186` (already pointing at `origin/master`). All commits land here, fast-forward only.
- All commits use Conventional Commits format. `fix(qbe):`, `test(qbe):`, etc.
- After every task: run `cargo test --bin sb test_examples` and `cargo test --lib`. Both must be green before committing.
- Manual end-to-end check after Task 1:
  ```bash
  cargo run -- --target qbe run examples/hello_world.sb; echo "exit=$?"
  ```
  Expected: `Hello World\n` followed by `exit=0` (was `exit=12`).

---

## File Structure

Files this plan touches.

**Modify:**
- `src/generator/qbe.rs` — `generate_function`, the only place. Add a `void_main_returns_zero` helper or inline the special case.
- `src/generator/tests/qbe_tests.rs` — add unit test that asserts the SSA emits `function w $main()` and `ret 0` for void source main.
- `src/tests/test_examples.rs` — replace `compile_and_run_qbe` with a stricter version that asserts exit-code 0 and (per-example) optional golden-stdout match. Remove the workaround comment about void main.
- `src/tests/test_examples/expected_qbe/<name>.txt` — new directory with golden stdout for examples we can fully verify (everything except `args`, which depends on argv that the harness doesn't control). Optional per-example.

**Touch only if Task 3-A is chosen (see decision point below):**
- `examples/bubblesort.sb` — replace `println(arr)` with an explicit `int_to_str(...)` chain so the example correctly prints `1,2,3,4,5`.

OR if Task 3-B is chosen:
- `src/parser/typechecker.rs` (or wherever the existing typechecker lives — to be confirmed during implementation) — reject `println(int[])` at parse/typecheck time.

OR if Task 3-C is chosen:
- `examples/bubblesort.sb` — leave as-is, but mark in `compile_and_run_qbe` as "skip stdout check" with a recorded reason. Not preferred but acceptable.

---

## Task 1: Void `main()` exits 0

**Files:**
- Modify: `src/generator/qbe.rs:490-574` (`generate_function`)
- Modify: `src/generator/tests/qbe_tests.rs` — add `test_void_main_returns_zero`

### Background

Master's `generate_function` (`src/generator/qbe.rs:490-574`) computes the QBE return type from `func.ret_type` (with a fallback inference for inline-bodied functions). For a void source `main`, `func.ret_type` is `None` and `effective_ret` is also `None`, producing `return_ty = None`. Then on line 562–573 the implicit-return is emitted as `qbe::Instr::Ret(None)` (bare `ret`). At runtime, that's whatever's in `w0`.

The fix is local: when `func.name == "main"` and `effective_ret` is `None`, force the function's QBE return type to `Some(qbe::Type::Word)` and emit `Ret(Some(Const(0)))` for the implicit return path. Existing explicit `Statement::Return(None)` inside main also needs to become `ret 0` — handled by also changing the `Statement::Return` codegen for that case OR (cleaner) by an AST-level transform that prepends `return 0` to the bottom of any void main body before codegen.

We pick the **codegen-level fix** because (a) it's local to `generate_function`, (b) avoids touching the AST transform layer, (c) it's the right *layer* — `main`'s ABI is a property of the produced executable, not the source language.

### Steps (TDD)

- [ ] **Step 1.1: Write the failing unit test**

Append to `src/generator/tests/qbe_tests.rs` inside the `mod tests { ... }` block:

```rust
#[test]
fn test_void_main_returns_zero() {
    let func = create_function("main", None, create_block_stmt(vec![]));
    let module = create_module(vec![func], Vec::new());
    let result = QbeGenerator::generate(module).unwrap();
    let result_norm = normalize_qbe(&result);
    assert!(
        result_norm.contains("export function w $main(")
            // qbe-rs v4 may format args either as `(w %argc, l %argv)` or `()`
            // depending on whether main() takes argv; we don't care here, only
            // that the return type is Word.
        ,
        "void main should be declared 'function w $main(...)':\n{}",
        result_norm
    );
    assert!(
        result_norm.contains("ret 0"),
        "void main should explicitly `ret 0`:\n{}",
        result_norm
    );
}
```

If `create_function` / `create_module` helpers no longer exist on master, recreate them inline using the public AST types, OR locate the equivalent test helper currently in use. (Master's `qbe_tests.rs` has been substantially rewritten — implementer should confirm.)

- [ ] **Step 1.2: Run the test, confirm it fails**

```bash
cargo test --bin sb generator::tests::qbe_tests::tests::test_void_main_returns_zero -- --nocapture
```
Expected: FAIL — current code produces `export function $main(...)` and `ret`.

- [ ] **Step 1.3: Apply the codegen fix**

In `src/generator/qbe.rs`, locate `generate_function` (around line 490). Make these surgical changes:

(a) **After `let effective_ret = ...`** and **before `let return_ty = match ...`**, insert:

```rust
        // Special-case `main`: when the source declared no return type, the
        // OS-level exit code must still be deterministic. We declare the QBE
        // function with Word return and force `ret 0` on every implicit
        // return path. Without this, `main`'s exit code is whatever was in
        // w0 from the last call (typically `printf`'s byte-count return).
        let is_void_main = func.name == "main" && effective_ret.is_none();
```

(b) **Modify the `return_ty` computation** to honor `is_void_main`:

```rust
        let return_ty = if is_void_main {
            Some(qbe::Type::Word)
        } else {
            match &effective_ret {
                Some(Type::Struct(_)) => Some(qbe::Type::Long),
                Some(ty) => Some(self.get_type(ty.to_owned())?.into_abi()),
                None => None,
            }
        };
```

(c) **Modify the implicit-return block** at the end of the function (around line 562):

Replace:
```rust
        if !returns {
            if func.ret_type.is_none() || last_block_empty {
                qfunc.add_instr(qbe::Instr::Ret(None));
            } else {
                return Err(format!(
                    "Function '{}' does not return in all code paths",
                    &func.name
                ));
            }
        }
```

with:
```rust
        if !returns {
            if is_void_main {
                qfunc.add_instr(qbe::Instr::Ret(Some(qbe::Value::Const(0))));
            } else if func.ret_type.is_none() || last_block_empty {
                qfunc.add_instr(qbe::Instr::Ret(None));
            } else {
                return Err(format!(
                    "Function '{}' does not return in all code paths",
                    &func.name
                ));
            }
        }
```

(d) **Handle explicit `Statement::Return(None)` inside void main.** Locate `Statement::Return` in `generate_statement` (around line 758). Add a check for "we're inside void main" — but `generate_statement` doesn't currently know which function it's in. Two options:

  - Option (i): track a `current_fn_is_void_main: bool` flag on `QbeGenerator` (push/pop around `generate_function`). Cheap, no API churn.
  - Option (ii): post-process `qfunc.blocks` after `generate_statement` returns and rewrite any `Ret(None)` to `Ret(Some(Const(0)))` when `is_void_main`.

Pick **option (ii)** — no state, single point of intervention, easy to audit. After `self.generate_statement(&mut qfunc, &func.body)?;` and before the `let returns = ...` block, insert:

```rust
        if is_void_main {
            // Rewrite explicit `return` (with no value) inside void main to
            // `ret 0` so the OS exit code is deterministic.
            for block in qfunc.blocks.iter_mut() {
                for item in block.items.iter_mut() {
                    if let qbe::BlockItem::Statement(qbe::Statement::Volatile(qbe::Instr::Ret(ref mut val))) = item {
                        if val.is_none() {
                            *val = Some(qbe::Value::Const(0));
                        }
                    }
                }
            }
        }
```

(qbe-rs v4 may have different field/variant access patterns. Implementer should adjust if the exact match doesn't compile, e.g. via `block.items_mut()` if blocks are accessed via methods rather than fields, or via a destructuring `Instr` if `Ret` has been renamed.)

- [ ] **Step 1.4: Run the unit test, confirm it passes**

```bash
cargo test --bin sb generator::tests::qbe_tests::tests::test_void_main_returns_zero
```
Expected: PASS.

- [ ] **Step 1.5: Manually verify all 6 affected examples now exit 0**

```bash
cargo build --bin sb
for ex in hello_world greeter leapyear fib ackermann loops; do
    cargo run --quiet -- --target qbe build "examples/${ex}.sb" -o "/tmp/${ex}.ssa"
    qbe -o "/tmp/${ex}.s" "/tmp/${ex}.ssa"
    cc -o "/tmp/${ex}.exe" "/tmp/${ex}.s" builtin/builtin_qbe.c
    "/tmp/${ex}.exe" >/dev/null 2>&1
    echo "${ex}: exit=$?"
done
```
Expected: every line reports `exit=0`.

- [ ] **Step 1.6: Run all unit tests, confirm no regression**

```bash
cargo test --bin sb generator::tests::qbe_tests
cargo test --lib 2>&1 | tail -5
```
Expected: green across the board.

- [ ] **Step 1.7: Commit**

```bash
git add src/generator/qbe.rs src/generator/tests/qbe_tests.rs
git commit -m "fix(qbe): emit 'ret 0' for void main so OS exit code is deterministic"
```

---

## Task 2: Tighten `compile_and_run_qbe` to lock in Task 1

**Files:**
- Modify: `src/tests/test_examples.rs:165-229` (`compile_and_run_qbe`)
- Create: `src/tests/test_examples/expected_qbe/<name>.txt` for each example with deterministic stdout (everything except `args`, which depends on argv).

### Background

Master's `compile_and_run_qbe` only asserts `execution.status.code().is_some()` (process wasn't killed by signal). We tighten it to:

1. Assert exit code is exactly 0 (now valid post-Task-1).
2. If a golden stdout file exists at `src/tests/test_examples/expected_qbe/<name>.txt`, assert exact match. If no golden file exists, fall back to the existing "doesn't contain FAIL" check (so newly-added examples without goldens don't immediately break the suite).

This makes Task 1's fix non-regressable and surfaces the bubblesort defect (which Task 3 then handles).

### Steps

- [ ] **Step 2.1: Create the golden output directory and files**

```bash
mkdir -p src/tests/test_examples/expected_qbe
```

Create one file per example with deterministic stdout. Each file is the *exact* expected stdout, including the trailing newline:

`src/tests/test_examples/expected_qbe/hello_world.txt`:
```
Hello World
```

`src/tests/test_examples/expected_qbe/greeter.txt`:
```
Hello World
```

`src/tests/test_examples/expected_qbe/leapyear.txt`:
```
Leap year
```

`src/tests/test_examples/expected_qbe/fib.txt`:
```
55
```

`src/tests/test_examples/expected_qbe/ackermann.txt`:
```
61
```

`src/tests/test_examples/expected_qbe/loops.txt`:
```
One
Two
Three
Apple
Strawberry
Orange
```

`src/tests/test_examples/expected_qbe/sandbox.txt`:
(empty — zero bytes; `sandbox.sb` is `fn main(): int { return 0 }`.)

**Do NOT create one for `args`** — its stdout depends on argv which the harness doesn't pass. **Do NOT create one for `bubblesort` yet** — its stdout is wrong today (the whole point of Task 3).

- [ ] **Step 2.2: Tighten `compile_and_run_qbe`**

In `src/tests/test_examples.rs`, replace `compile_and_run_qbe` (around lines 165–229) with:

```rust
/// Compile a single .sb file through the full QBE pipeline and execute it.
/// Asserts:
///   - the binary exits with code 0,
///   - if a golden file exists at src/tests/test_examples/expected_qbe/<name>.txt,
///     stdout matches it exactly; otherwise stdout must not contain "FAIL".
///
/// Pipeline: .sb → (antimony) → .ssa → (qbe) → .s → (gcc) → binary → run
fn compile_and_run_qbe(in_file: &std::path::Path, dir_out: &std::path::Path) -> Result<(), Error> {
    let dir = std::env::current_dir().unwrap();

    let base_name = in_file.file_stem().unwrap().to_string_lossy().into_owned();
    let ssa_file = dir_out.join(format!("{}.ssa", base_name));
    let asm_file = dir_out.join(format!("{}.s", base_name));
    let bin_file = dir_out.join(&base_name);

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

    let execution = Command::new(&bin_file).output()?;
    let stdout = String::from_utf8_lossy(&execution.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&execution.stderr).into_owned();

    assert_eq!(
        execution.status.code(),
        Some(0),
        "Binary {:?} exited with non-zero code {:?}\nstdout: {}\nstderr: {}",
        &bin_file,
        execution.status.code(),
        stdout,
        stderr
    );

    let golden_path = dir
        .join("src/tests/test_examples/expected_qbe")
        .join(format!("{}.txt", base_name));
    if golden_path.exists() {
        let expected = std::fs::read_to_string(&golden_path)?;
        assert_eq!(
            stdout, expected,
            "stdout mismatch for {:?}\n--- expected ---\n{}\n--- actual ---\n{}\n",
            &bin_file, expected, stdout,
        );
    } else {
        assert!(
            !stdout.contains("FAIL"),
            "Binary stdout contains FAIL for {:?}\nstdout: {}\nstderr: {}",
            &bin_file,
            stdout,
            stderr
        );
    }

    Ok(())
}
```

- [ ] **Step 2.3: Run the tightened example test**

```bash
cargo test --bin sb test_examples_qbe -- --nocapture 2>&1 | tail -40
```
Expected: 8 examples PASS (all that have determinable stdout, which now exit 0 thanks to Task 1, AND `args` which doesn't have a golden file — falls through to the "no FAIL" check). `bubblesort` FAILS — empty stdout vs. (no golden, no failure case applies)... wait, since we didn't create `bubblesort.txt`, it falls back to the "doesn't contain FAIL" branch, and empty stdout doesn't contain FAIL — so it'd still pass with exit 0.

Actually `bubblesort` exits **2** today, so the new exit-code-0 assertion **catches it**. Expected: 8 PASS, 1 FAIL (`bubblesort` — exit 2, stdout empty).

If `bubblesort` somehow ends up at exit 0 in your run (e.g., if some other commit on master fixed it after this plan was written), proceed to Task 3 and re-evaluate.

- [ ] **Step 2.4: Commit**

```bash
git add src/tests/test_examples.rs src/tests/test_examples/
git commit -m "test(qbe): assert exit=0 and golden-stdout per example in compile_and_run_qbe"
```

---

## Task 3: Resolve `bubblesort` (decision point)

**Decision required from project owner before this task starts.** Three valid options; pick one. No "do nothing" — the failing test from Task 2 must be addressed.

### Option 3-A: Fix the example to print correctly via `int_to_str`

**Rationale:** Smallest backend change (none). Demonstrates the *actual* shape of valid array printing in current Antimony — a chain of `int_to_str(arr[i])` calls joined by `","`. Educational for users reading `examples/`. Doesn't paper over a real backend gap (the lack of array-to-string conversion).

**Cost:** changes the example. The original was relying on JS's `Array.prototype.toString()` implicit coercion — a behavior Antimony's JS backend inherits but the QBE backend doesn't have. Arguable that the example was always backend-dependent.

**Files:** `examples/bubblesort.sb`, `src/tests/test_examples/expected_qbe/bubblesort.txt` (new golden).

**Replacement** (`examples/bubblesort.sb`):

```rust
import "../lib/string"

fn main() {
    let arr = [2, 5, 3, 1, 4]
    let n = len(arr)

    let c = 0
    while c < n {
        let d = 0
        while d < n - c - 1 {
            let current = arr[d]
            let next = arr[d+1]
            if current > next {
                let swap = arr[d]
                arr[d]   = arr[d+1]
                arr[d+1] = swap
            }
            d += 1
        }
        c += 1
    }

    let i = 0
    let out = ""
    while i < n {
        if i > 0 {
            out = out + ","
        }
        out = out + int_to_str(arr[i])
        i += 1
    }
    println(out)
}
```

(Confirm `import "../lib/string"` is the actual module-import syntax in master's parser; if the import path needs to be absolute or named differently, adjust during implementation.)

**Golden** (`src/tests/test_examples/expected_qbe/bubblesort.txt`):
```
1,2,3,4,5
```

### Option 3-B: Add a typecheck rejection for `println(non-string)`

**Rationale:** Catches the underlying backend gap once, for all examples. Future code that does `println(some_int)` or `println(some_arr)` gets a compile error instead of silent wrong output. Matches the strictness of the rest of the language (Antimony's typechecker already rejects e.g. struct-decl errors).

**Cost:** larger surface — requires finding/extending master's typechecker. Risks rejecting other valid programs in the wider corpus (`tests/`, `tests/qbe/*.sb`).

**Files:** to be confirmed during implementation. Plausible locations: `src/parser/typechecker.rs` (if it exists), `src/parser/infer.rs`, or one of the AST transform passes in `src/ast/transform.rs`.

**Out-of-scope risk:** This option may also need updates to the `examples/bubblesort.sb` source if we want the test to pass — the strict rejection means the example *can't compile* until the source is fixed. So 3-B effectively forces 3-A as a follow-up. Recommend skipping 3-B unless project owner explicitly wants the strictness.

### Option 3-C: Mark `bubblesort` as known-broken in the harness

**Rationale:** Acknowledges the gap without changing example or backend. Useful only if (a) we plan to land array-to-string conversion in a follow-up plan and want to keep the example's original shape, or (b) we want the test suite to report this as a known regression rather than a hard failure.

**Cost:** Zero technical debt avoided; encodes the gap in a hard-to-find skiplist. **Not recommended** unless option 3-D (add real array-to-string runtime) is being planned for the immediate next plan.

**Files:** `src/tests/test_examples.rs` — extend `compile_and_run_qbe` with a per-example "expect this to fail" mechanism. **Don't choose this option** without an explicit plan to remove the skiplist.

### Recommendation

**Pick 3-A.** Smallest blast radius, makes the example work correctly today, and the rewritten loop is a useful demonstration of Antimony idioms (the original `println(arr)` was a JS-flavored shortcut). Comments in the source can still note: *"// JS backend supports `println(arr)` directly via array.toString(); QBE's println takes string only."*

If the project owner wants Antimony to support `println(arr)` everywhere, a follow-up plan should land `_array_to_str_int` (and friends) in `builtin/builtin_qbe.c`, with a small JS-side noop, and a backend dispatch for `println(<int_array_typed>)`. Out of scope here.

### Steps for Option 3-A

(Skip if project owner picks 3-B or 3-C — those become their own plans.)

- [ ] **Step 3.1: Replace `examples/bubblesort.sb`** with the version under "Option 3-A" above. Keep the comment in source noting why the formatting is explicit.

- [ ] **Step 3.2: Verify JS backend still works**

```bash
cargo run -- run examples/bubblesort.sb
```
Expected: stdout `1,2,3,4,5` (matches the new explicit formatting).

- [ ] **Step 3.3: Verify QBE backend matches**

```bash
cargo run -- --target qbe build examples/bubblesort.sb -o /tmp/bs.ssa
qbe -o /tmp/bs.s /tmp/bs.ssa
cc -o /tmp/bs.exe /tmp/bs.s builtin/builtin_qbe.c
/tmp/bs.exe; echo "exit=$?"
```
Expected: stdout `1,2,3,4,5` and `exit=0`.

- [ ] **Step 3.4: Add the golden file**

`src/tests/test_examples/expected_qbe/bubblesort.txt`:
```
1,2,3,4,5
```

- [ ] **Step 3.5: Run the full test suite**

```bash
cargo test --bin sb test_examples 2>&1 | tail -10
```
Expected: all green (`test_examples_js`, `test_examples_qbe`, `test_qbe_execution_tests`). The QBE example test now strictly verifies that all 9 examples produce exit 0 and (for the 8 with goldens) match exact stdout.

- [ ] **Step 3.6: Commit**

```bash
git add examples/bubblesort.sb src/tests/test_examples/expected_qbe/bubblesort.txt
git commit -m "feat(examples): bubblesort prints via explicit int_to_str (QBE-compatible)"
```

---

## Final Verification

- [ ] **Step F.1: Sanity-run all example tests**

```bash
cargo test --bin sb test_examples -- --test-threads=1 --nocapture 2>&1 | tail -20
```
Expected: 3 tests passing (`test_examples_js`, `test_examples_qbe`, `test_qbe_execution_tests`), no skipped.

- [ ] **Step F.2: Run the full test suite**

```bash
cargo test
```
Expected: all green.

- [ ] **Step F.3: Run each example manually one final time**

```bash
for ex in examples/*.sb; do
    name=$(basename "$ex" .sb)
    [ "$name" = "args" ] && continue   # argv-dependent
    echo "--- $name ---"
    cargo run --quiet -- --target qbe build "$ex" -o "/tmp/${name}.ssa"
    qbe -o "/tmp/${name}.s" "/tmp/${name}.ssa"
    cc -o "/tmp/${name}.exe" "/tmp/${name}.s" builtin/builtin_qbe.c
    "/tmp/${name}.exe"
    echo "exit=$?"
done
```
Expected: every example produces its golden stdout and exits 0.

---

## Self-Review

**Spec coverage:**
| Survey finding | Addressed by task |
|---|---|
| Void `main()` returns `printf`'s last byte count | Task 1 |
| `bubblesort` silent wrong-codegen | Task 3 (option 3-A recommended) |
| `compile_and_run_qbe` test runner is too lax | Task 2 |

**Placeholder scan:** No "TBD" or "implement later" — every code-changing step has the actual code.

**Type consistency:** `is_void_main: bool` introduced in Task 1 and used identically in three sites within `generate_function`. Golden file directory `src/tests/test_examples/expected_qbe/` referenced consistently in Tasks 2 and 3. Golden filenames match example basenames.

**Caveats / things that might surprise the implementer:**
- **qbe-rs v4 API drift.** Master is on `qbe = "4.0.0"`. The exact field/variant access in `qbe::BlockItem`, `qbe::Statement::Volatile`, `qbe::Instr::Ret(_)` may differ from v3. The implementer should `cargo doc --open --package qbe` if the post-process loop in Task 1 Step 1.3(d) doesn't compile cleanly, and adjust to whatever idiom v4 expects. The intent (rewrite `Ret(None)` → `Ret(Some(Const(0)))` inside void main) is what matters; the syntax is mechanical.
- **`infer_fn_return_type` is master's existing helper.** Don't duplicate or replace it — just gate it behind `is_void_main`.
- **The `last_block_empty` branch in master's existing code** is meant to handle "all paths return but the synthetic terminator block is empty" — it's correct for non-main typed functions and we keep it intact.
- **`tests/qbe/*.sb` self-checking files** are unaffected by Task 2 — they go through the `_checked` runner, not `compile_and_run_qbe`.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-04-27-qbe-void-main-and-test-tightening.md`. Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks. With only 3 tasks (4 if Task 3-A is the chosen option), this is a single short chunk.
2. **Inline Execution** — Execute tasks in this session with checkpoints.

Which approach? And: which Task 3 option (A / B / C)?
