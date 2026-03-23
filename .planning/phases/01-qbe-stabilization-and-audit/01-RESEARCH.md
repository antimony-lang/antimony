# Phase 1: QBE Stabilization and Audit - Research

**Researched:** 2026-03-23
**Domain:** QBE backend testing, unsafe code elimination, language feature gap analysis
**Confidence:** HIGH

## Summary

Phase 1 is an audit and test-hardening phase, not a feature-building phase. The three requirements (STAB-01, STAB-02, STAB-03) decompose cleanly into: (1) build an execution test harness for QBE-compiled binaries, (2) fix the `qbe` Rust crate's lifetime constraints to eliminate two `unsafe { std::mem::transmute }` calls, and (3) systematically test every language feature against QBE codegen and document gaps.

The existing codebase provides a solid foundation. The `compile_and_run_qbe()` function in `src/tests/test_examples.rs` already implements the full QBE pipeline (.sb -> .ssa -> .s -> binary -> execute) and can be extended to discover and run test programs from a new `tests/qbe/` directory. The user owns the `qbe` crate, making the upstream lifetime fix a direct change with no third-party dependency. The language feature set is well-defined by the LAST (Low-level AST) enum variants, providing a finite, enumerable list of features to audit.

**Primary recommendation:** Start with the upstream `qbe` crate lifetime fix (unblocks STAB-02), then build the test harness (STAB-01), then run the systematic feature sweep to produce the gap inventory (STAB-03).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Expand the integration test suite with new .sb programs that compile and run through the full QBE pipeline (`.sb` -> `.ssa` -> `.s` -> binary -> execution). Keep existing 48 IL snapshot unit tests as-is.
- **D-02:** Execution tests validate both stdout content AND exit code (most thorough).
- **D-03:** New QBE execution tests live in `tests/qbe/` directory, separate from `examples/` (which stays clean for documentation).
- **D-04:** Fix the lifetime issue upstream in the `qbe` crate (user owns the crate). Relax lifetime constraints or add an owned type variant so the two `unsafe { std::mem::transmute }` calls (lines 201 and 1632 of `src/generator/qbe.rs`) can be removed. No fallback needed -- user will merge the fix.
- **D-05:** After the crate fix is published, update `Cargo.toml` to the new version and remove both transmutes from the generator.
- **D-06:** Gap inventory is a feature checklist with severity: table listing every language feature, QBE status (pass/fail/partial), and severity (blocks bootstrap / needed later / nice-to-have).
- **D-07:** Gap inventory lives at `.planning/phases/01-qbe-stabilization-and-audit/QBE-GAPS.md` as a phase artifact.
- **D-08:** Each gap is cross-referenced to the roadmap phase that resolves it (e.g., "string indexing -> Phase 2: Runtime Primitives"). Makes the inventory a planning tool, not just a list.
- **D-09:** Systematic language feature sweep -- write a test program for every language feature (each type, control flow construct, expression kind, structs, arrays, methods, etc.) to get the full picture for the gap inventory.
- **D-10:** One .sb test file per feature (e.g., `test_structs.sb`, `test_while_loops.sb`, `test_string_ops.sb`). Easy to isolate failures.
- **D-11:** Test programs are self-checking: each .sb does its own assertions, prints 'PASS' or 'FAIL: reason', exits 0 or 1. The test harness just checks exit code and scans for FAIL.

### Claude's Discretion
- Order of language features to test (can be derived from the AST/parser feature set)
- Internal structure of the test harness (how `test_examples_qbe` pattern is extended)
- How to enumerate language features systematically (walk the AST enum variants, parser capabilities, etc.)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| STAB-01 | End-to-end execution tests exist -- programs are compiled via QBE, linked, and executed (not just IL text-checked) | Existing `compile_and_run_qbe()` pattern in `src/tests/test_examples.rs` provides the pipeline template. New test harness discovers .sb files in `tests/qbe/`, compiles through full pipeline, checks exit code AND scans stdout for FAIL. |
| STAB-02 | Unsafe transmute UB in QBE codegen is resolved | Two transmutes at lines 201 and 1632 of `src/generator/qbe.rs`. Root cause: `qbe::Type::Aggregate(&'a TypeDef<'a>)` borrows a `TypeDef` but the generator needs `'static` lifetime. Fix: add owned variant to `qbe::Type` in the upstream crate (user-owned). |
| STAB-03 | Formal gap inventory completed -- every language feature is tested for correct QBE codegen and gaps are documented | LAST enum variants define the complete feature set: 9 Statement variants, 12 Expression variants, 18 BinOp variants, 6 Type variants. Each needs a test program in `tests/qbe/`. Results feed into QBE-GAPS.md. |
</phase_requirements>

## Architecture Patterns

### Existing QBE Pipeline (reuse as-is)

```
.sb --[cargo run -- --target qbe build]--> .ssa --[qbe]--> .s --[gcc + builtin_qbe.c]--> binary --[execute]--> check
```

This pipeline is already implemented in `compile_and_run_qbe()` at `src/tests/test_examples.rs:71-138`. The function handles each stage and asserts success at each step.

### Test Harness Extension Pattern

The existing harness has a limitation: `compile_and_run_qbe()` only checks that the binary does not crash (signal). It does NOT check exit code for success (0) because void `main()` functions do not emit `ret 0` in the current QBE backend. The new test harness for `tests/qbe/` must:

1. Check exit code IS 0 (not just "any code" like current code)
2. Capture stdout and scan for "FAIL" substring
3. Each `.sb` test must call `exit(0)` explicitly at the end of `main()` to work around the void-main return code issue

**Critical detail from `src/tests/test_examples.rs:129`:**
```rust
// Note: void main() may return a non-zero exit code in QBE since
// the backend doesn't yet emit `ret 0` for void functions, so we
// only check that the process wasn't killed by a signal.
let execution = Command::new(&bin_file).output()?;
assert!(execution.status.code().is_some(), ...);
```

Self-checking test programs MUST call `exit(0)` at the end of a successful run to produce a reliable exit code.

### Recommended Test Program Structure

```
tests/qbe/
  test_int_arithmetic.sb    # Int type + arithmetic BinOps
  test_int_comparison.sb    # Int comparison operators
  test_booleans.sb          # Bool type + logical operators
  test_strings.sb           # String type + concatenation
  test_variables.sb         # Let declarations, assignment
  test_if_else.sb           # If/else/else-if chains
  test_while_loops.sb       # While loops, break, continue
  test_for_loops.sb         # For-in loops
  test_functions.sb         # Function calls, return values, recursion
  test_arrays.sb            # Array creation, access, assignment, len()
  test_structs.sb           # Struct init, field access, nested structs
  test_methods.sb           # Struct methods, self keyword
  test_match.sb             # Match statements (desugared to if-else)
  test_string_ops.sb        # String concat, int_to_str, str_len
  test_any_type.sb          # Any type parameter passing
  test_imports.sb           # Module imports (needs submodule dir)
  test_compound_assign.sb   # +=, -=, *=, /= operators
  test_nested_expressions.sb # Complex nested BinOps, field access chains
```

### Self-Checking Test Program Template

```
// test_<feature>.sb
fn main() {
    // Test case 1
    let x = 1 + 2
    if x != 3 {
        println("FAIL: 1 + 2 should equal 3")
        exit(1)
    }

    // Test case 2 ...

    println("PASS")
    exit(0)
}
```

Key points:
- Each test calls `exit(0)` on success (works around void-main issue)
- Each test calls `exit(1)` with a FAIL message on failure
- The `exit()` and `println()` builtins are available via `builtin_qbe.c` (linked automatically by gcc in the pipeline)
- The stdlib `assert.sb` is NOT automatically available -- tests should inline their own checks or the harness must include the lib path

### Anti-Patterns to Avoid
- **Relying on exit code from void main():** QBE backend does not emit `ret 0` for void functions. Always use explicit `exit(0)`.
- **Importing stdlib in execution tests:** The current `compile_and_run_qbe()` does not pass lib path flags. Tests should be self-contained using only builtins (`println`, `exit`, `_int_to_str`, `_str_concat`, `_strlen`, `_parse_int`).
- **Testing match statements directly against QBE:** Match is a HAST construct lowered to if-else chains by the transformer. QBE tests should still include match to verify the full pipeline, but know that failures may be in the transform layer, not the QBE generator.

## Complete Language Feature Inventory (for systematic sweep)

### From Low-Level AST (what QBE generator actually receives)

**Types** (from `src/ast/types.rs`):
| Type | Description | Builtins Available |
|------|-------------|-------------------|
| `Int` | 64-bit integer (maps to QBE Long) | `_int_to_str()` |
| `Str` | String (pointer, maps to QBE Long) | `_printf()`, `_str_concat()`, `_strlen()` |
| `Bool` | Boolean (maps to QBE Word) | none specific |
| `Any` | Untyped/polymorphic | none specific |
| `Array(T, size)` | Fixed-size arrays | `len()` via intrinsic |
| `Struct(name)` | User-defined structs | none |

**Statements** (from `src/ast/last.rs`):
| Statement | Notes |
|-----------|-------|
| `Block` | Scoped statement list |
| `Declare` | Variable declaration with optional init |
| `Assign` | Assignment to variable or field |
| `Return` | Return value or void |
| `If` | Conditional with optional else |
| `While` | While loop |
| `For` | For-in loop (iterates arrays) |
| `Break` | Loop break |
| `Continue` | Loop continue |
| `Exp` | Expression statement (side effects) |

**Expressions** (from `src/ast/last.rs`):
| Expression | Notes |
|------------|-------|
| `Int(usize)` | Integer literal |
| `Str(String)` | String literal |
| `Bool(bool)` | Boolean literal |
| `Selff` | Self reference in methods |
| `Array { capacity, elements }` | Array literal |
| `FunctionCall { fn_name, args }` | Function call |
| `Variable(String)` | Variable reference |
| `ArrayAccess { name, index }` | Array element access |
| `BinOp { lhs, op, rhs }` | Binary operation |
| `StructInitialization { name, fields }` | Struct constructor |
| `FieldAccess { expr, field }` | Struct field access |

**BinOps** (18 total from `src/ast/last.rs`):
| BinOp | Symbol |
|-------|--------|
| Addition | `+` |
| Subtraction | `-` |
| Multiplication | `*` |
| Division | `/` |
| Modulus | `%` |
| LessThan | `<` |
| LessThanOrEqual | `<=` |
| GreaterThan | `>` |
| GreaterThanOrEqual | `>=` |
| Equal | `==` |
| NotEqual | `!=` |
| And | `&&` |
| Or | `\|\|` |
| AddAssign | `+=` |
| SubtractAssign | `-=` |
| MultiplyAssign | `*=` |
| DivideAssign | `/=` |

**High-level constructs (lowered before QBE sees them):**
| HAST Construct | Lowered To |
|----------------|-----------|
| Match statement | If-else chain (by `src/ast/transform.rs`) |
| For-in on range | While loop (conceptually) |

### Existing Coverage (what is already tested)

The existing `tests/` directory runs through QBE via `test_testcases_qbe()` and exercises:
- Basic conditionals (if/else/else-if)
- Match statements (desugared)
- Functions (calls, return values)
- Numbers (literals, hex, binary, octal, compound assignment)
- Structs (init, field access, nested, methods, self)
- Types (any type)
- Unicode strings
- Imports (module system)
- Arrays (via `tests/arrays.sb` -- standalone file, not part of main.sb suite)

The existing `examples/` directory runs through QBE via `test_examples_qbe()` and exercises:
- hello_world (basic string output)
- fib (recursion, comparison, arithmetic)
- bubblesort (arrays, while loops, comparison, swap)
- ackermann, greeter, leapyear, loops, sandbox

**BUT:** The existing `test_testcases_qbe()` only verifies the binary doesn't crash (signal check). It does NOT verify correctness (exit code or stdout). This is the core gap STAB-01 addresses.

## Transmute Analysis (STAB-02)

### Root Cause

The `qbe` crate (version 3.0.0, user-owned, published on crates.io) defines:

```rust
pub enum Type<'a> {
    Word, Long, Single, Double, Zero,
    Byte, SignedByte, UnsignedByte,
    Halfword, SignedHalfword, UnsignedHalfword,
    Aggregate(&'a TypeDef<'a>),  // <-- borrows TypeDef
}
```

The `Aggregate` variant holds a borrowed reference to a `TypeDef`. The Antimony QBE generator stores `TypeDef`s in a `Vec<Rc<TypeDef<'static>>>` and needs to create `Type::Aggregate` values that reference these. Since the `Rc` ensures the `TypeDef` outlives the reference, the transmute is "safe in practice" but technically UB because it lies about lifetimes.

### Two Transmute Locations

1. **Line 201** (`generate()` method, struct registration): Creates `Type::Aggregate` referencing a just-pushed `Rc<TypeDef>` in `self.typedefs`.
2. **Line 1632** (array type creation): Same pattern for array type definitions.

### Fix Strategy (per D-04, D-05)

Modify the `qbe` crate to support owned types. Options (Claude's discretion for implementation detail):
- Add an `OwnedAggregate(Rc<TypeDef<'static>>)` variant to `Type`
- Change `Aggregate` to use `Rc` instead of a reference
- Add a parallel `OwnedType` enum with conversions

After publishing the updated crate, update `Cargo.toml` and remove both transmutes. The `Rc<TypeDef>` pattern already in use maps naturally to an owned variant.

### Confidence: HIGH
The user owns the crate. The fix is straightforward (change a reference to an owned/Rc type). No external dependencies or approvals needed.

## Common Pitfalls

### Pitfall 1: Void Main Exit Code
**What goes wrong:** QBE-compiled binaries with `void main()` return unpredictable exit codes because the backend does not emit `ret 0` for void functions.
**Why it happens:** The QBE generator does not add an implicit `ret 0` at the end of functions with no return type.
**How to avoid:** All self-checking test programs must call `exit(0)` explicitly on the success path. The test harness must check for exit code 0.
**Warning signs:** Tests that "pass" even when assertions fail, or tests that always fail with non-zero exit even when correct.

### Pitfall 2: Missing Stdlib in Test Programs
**What goes wrong:** Test programs that `import "stdlib/assert"` or other lib modules may fail because the test harness does not pass `--lib` flags to the compiler.
**Why it happens:** `compile_and_run_qbe()` invokes `cargo run -- --target qbe build <file> -o <out>` without library path configuration.
**How to avoid:** Self-checking tests should use only builtins available through `builtin_qbe.c` (printf, exit, int_to_str, str_concat, strlen, parse_int, read_line). Inline assertion logic directly.
**Warning signs:** "Module not found" errors during compilation of test programs.

### Pitfall 3: HashMap Iteration Order in Struct Fields
**What goes wrong:** `StructInitialization` stores fields in a `HashMap<String, Box<Expression>>`, which has non-deterministic iteration order. If the QBE generator processes fields in iteration order rather than definition order, struct layout may be wrong.
**Why it happens:** HashMap does not preserve insertion order.
**How to avoid:** The generator should (and does) use the struct definition order from `StructDef.fields` when calculating offsets, not the initialization HashMap keys. Test with multi-field structs where field order matters.
**Warning signs:** Struct field values appear swapped or corrupted.

### Pitfall 4: String Equality Comparison
**What goes wrong:** `==` on strings may compare pointer addresses instead of string content.
**Why it happens:** Strings are pointers (Long) in QBE. The `==` BinOp generates an integer comparison unless the generator has special string-comparison handling.
**How to avoid:** Test string equality specifically. Document as a gap if it fails (likely Phase 2 fix with strcmp).
**Warning signs:** String comparisons that should be true return false.

### Pitfall 5: Array Length Intrinsic vs Sentinel
**What goes wrong:** The stdlib `len()` function (in `lib/array.sb`) counts elements until a zero/falsy value. The QBE intrinsic `len()` reads a header at offset 0. These are different implementations.
**Why it happens:** Two different `len()` implementations exist -- the stdlib version and the QBE intrinsic.
**How to avoid:** Test `len()` on arrays that contain zero elements to see which implementation wins and whether it is correct.
**Warning signs:** `len()` returns wrong values, especially for arrays containing 0.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| QBE pipeline orchestration | Custom shell scripts per test | Extend `compile_and_run_qbe()` in Rust | Already handles all stages with proper error messages |
| Test discovery | Manual test file lists | `std::fs::read_dir("tests/qbe/")` | Consistent with existing `test_examples_qbe` pattern |
| Assertion in .sb programs | Complex assertion framework | Inline if/println/exit pattern | Builtins are sufficient; no import machinery needed |
| Struct layout calculation | Manual offset math | Existing `generate_struct()` with alignment logic | Already handles padding and alignment correctly |

## Code Examples

### Extending the Test Harness (based on existing pattern)

```rust
// Source: src/tests/test_examples.rs (existing compile_and_run_qbe pattern)
// New function for self-checking tests in tests/qbe/
fn compile_and_run_qbe_checked(in_file: &Path, dir_out: &Path) -> Result<(), Error> {
    // ... same pipeline as compile_and_run_qbe() ...

    // NEW: Check exit code is exactly 0
    let execution = Command::new(&bin_file).output()?;
    assert!(
        execution.status.success(),
        "Test failed for {:?} (exit code {:?}):\nstdout: {}\nstderr: {}",
        in_file,
        execution.status.code(),
        String::from_utf8_lossy(&execution.stdout),
        String::from_utf8_lossy(&execution.stderr),
    );

    // NEW: Scan stdout for FAIL
    let stdout = String::from_utf8_lossy(&execution.stdout);
    assert!(
        !stdout.contains("FAIL"),
        "Test reported failure for {:?}:\n{}",
        in_file, stdout,
    );

    Ok(())
}
```

### Self-Checking Test Program (.sb)

```
// tests/qbe/test_int_arithmetic.sb
fn main() {
    // Addition
    let a = 1 + 2
    if a != 3 {
        println("FAIL: 1 + 2 != 3")
        exit(1)
    }

    // Subtraction
    let b = 5 - 3
    if b != 2 {
        println("FAIL: 5 - 3 != 2")
        exit(1)
    }

    println("PASS")
    exit(0)
}
```

### QBE Crate Type Fix Pattern

```rust
// In the qbe crate: add owned variant
pub enum Type<'a> {
    // ... existing variants ...
    Aggregate(&'a TypeDef<'a>),
    /// Owned aggregate type that does not borrow a TypeDef
    OwnedAggregate(std::rc::Rc<TypeDef<'static>>),
}
```

## Gap Inventory Template (QBE-GAPS.md)

```markdown
| Feature | Category | QBE Status | Severity | Resolving Phase | Notes |
|---------|----------|------------|----------|-----------------|-------|
| Int arithmetic | Expression/BinOp | PASS | - | - | + - * / % all work |
| String equality | Expression/BinOp | FAIL | Blocks bootstrap | Phase 2 | Compares pointers, needs strcmp |
| For-in loop | Statement | PARTIAL | Needed later | Phase 2 | Only works for int arrays |
```

Severity levels:
- **Blocks bootstrap**: Must work for the self-hosted compiler (Phase 4-5)
- **Needed later**: Required for Doom or full language, not bootstrap
- **Nice-to-have**: Would be good but not blocking any milestone

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test framework (cargo test) |
| Config file | None needed -- uses `#[test]` attributes in source |
| Quick run command | `cargo test test_qbe -- --nocapture` |
| Full suite command | `cargo test -- --nocapture` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| STAB-01 | QBE execution tests exist and run in CI | integration | `cargo test test_qbe_execution_tests -- --nocapture` | No -- Wave 0 |
| STAB-02 | No unsafe transmute in qbe.rs | unit (grep/compile check) | `cargo build 2>&1 && ! grep -c 'transmute' src/generator/qbe.rs` | No -- Wave 0 |
| STAB-03 | Gap inventory document exists and is complete | manual | Verify QBE-GAPS.md has entries for all LAST enum variants | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test test_qbe -- --nocapture` (run new QBE execution tests)
- **Per wave merge:** `cargo test -- --nocapture` (full suite including JS tests)
- **Phase gate:** Full suite green + QBE-GAPS.md covers all features

### Wave 0 Gaps
- [ ] `tests/qbe/` directory -- does not exist yet, needs creation
- [ ] Test harness function for `tests/qbe/` discovery -- extend `src/tests/test_examples.rs`
- [ ] At least one `.sb` self-checking test to validate the harness works

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/Cargo | Compiler build | Yes | 1.93.0 | -- |
| QBE | .ssa -> .s compilation | Yes | present at /opt/homebrew/bin/qbe | -- |
| GCC (clang) | .s -> binary linking | Yes | Apple clang 17.0.0 | -- |
| qbe crate | Rust QBE IL generation | Yes | 3.0.0 (Cargo.toml) | -- |

**Note:** Local environment has QBE and GCC available. CI also installs QBE from source (qbe-1.2 from c9x.me). All dependencies satisfied.

## Sources

### Primary (HIGH confidence)
- `src/generator/qbe.rs` -- Full QBE generator source (1,753 lines), transmute locations verified at lines 201 and 1632
- `src/tests/test_examples.rs` -- Existing test harness with `compile_and_run_qbe()` pattern
- `src/ast/last.rs` -- Complete LAST enum definitions (Statement, Expression, BinOp)
- `src/ast/hast.rs` -- Complete HAST enum definitions (HStatement, HExpression, HBinOp)
- `src/ast/types.rs` -- Type enum (Int, Str, Bool, Any, Array, Struct)
- `builtin/builtin_qbe.c` -- 7 C builtins available at link time
- `qbe` crate source at `~/.cargo/registry/src/.../qbe-3.0.0/src/lib.rs` -- Type<'a> lifetime definition verified

### Secondary (MEDIUM confidence)
- `.github/workflows/ci.yml` -- CI pipeline installs QBE 1.2 from source, runs `cargo test`

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- this is entirely internal tooling (Rust test framework, existing crate)
- Architecture: HIGH -- patterns directly derived from existing working code
- Pitfalls: HIGH -- identified from code inspection of actual implementations
- Gap inventory scope: HIGH -- derived from exhaustive AST enum inspection

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (stable -- no external dependency churn expected)
