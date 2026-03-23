---
phase: 01-qbe-stabilization-and-audit
plan: 01
subsystem: test-infrastructure
tags: [qbe, testing, harness, gap-inventory]
dependency_graph:
  requires: []
  provides: [qbe-execution-harness, qbe-test-programs]
  affects: [01-02-PLAN.md]
tech_stack:
  added: []
  patterns: [self-checking-test-programs, result-collecting-test-harness]
key_files:
  created:
    - tests/qbe/test_int_arithmetic.sb
    - tests/qbe/test_int_comparison.sb
    - tests/qbe/test_booleans.sb
    - tests/qbe/test_variables.sb
    - tests/qbe/test_if_else.sb
    - tests/qbe/test_while_loops.sb
    - tests/qbe/test_functions.sb
    - tests/qbe/test_strings.sb
  modified:
    - src/tests/test_examples.rs
decisions:
  - "compile_and_run_qbe_checked returns Result<(), String> not Result<(), Error> to allow per-file failure collection without aborting"
  - "test_qbe_execution_tests collects all failures and reports them rather than panicking on first failure -- all results are gap data"
metrics:
  duration: "~8 minutes"
  completed: "2026-03-23"
  tasks_completed: 2
  files_created: 8
  files_modified: 1
---

# Phase 1 Plan 01: QBE Execution Test Harness Summary

**One-liner:** QBE execution test harness with `compile_and_run_qbe_checked()` (exit code + stdout FAIL scan) and 8 self-checking `.sb` programs covering arithmetic, comparisons, booleans, variables, if/else, while loops, functions, and strings.

## What Was Built

### Task 1: QBE Execution Test Harness

Added `compile_and_run_qbe_checked()` to `src/tests/test_examples.rs`:
- Returns `Result<(), String>` (not `io::Error`) so failures are descriptive and collectable
- Runs the full pipeline: `.sb` -> `.ssa` -> `.s` -> binary -> execute
- Checks `execution.status.success()` (exit code == 0)
- Scans stdout for `"FAIL"` substring
- On failure, returns an `Err(String)` with stdout and stderr for debugging

Added `test_qbe_execution_tests()` Rust test:
- Discovers all `.sb` files in `tests/qbe/` via `read_dir`
- Runs each through `compile_and_run_qbe_checked()`
- Collects all results (no early abort on failure)
- Prints per-file PASS/FAIL and a summary at end
- Returns `Ok(())` always -- failures are printed as gap data, not test suite failures
- This allows `cargo test test_qbe_execution_tests` to succeed while capturing which programs fail

### Task 2: 8 Self-Checking Test Programs

All programs in `tests/qbe/` follow the convention: builtins only (no stdlib imports), `exit(0)` on success, `exit(1)` on failure, `println("PASS")` / `println("FAIL: reason")`.

| File | Feature Tested | Status |
|------|---------------|--------|
| test_int_arithmetic.sb | +, -, *, /, % | PASS |
| test_int_comparison.sb | <, <=, >, >=, ==, != | PASS |
| test_booleans.sb | bool literals, &&, \|\| | FAIL (gap) |
| test_variables.sb | let, typed let, reassign | PASS |
| test_if_else.sb | if, if/else, nested | PASS |
| test_while_loops.sb | while, break, continue, nested | PASS |
| test_functions.sb | fn, args, return, recursion | PASS |
| test_strings.sb | string lit, int_to_str, strlen | FAIL (gap) |

## Test Results

6/8 tests pass. 2 failures captured as gap data for Plan 02:

1. **test_booleans.sb** -- Binary exits with non-zero code. The boolean logic compiles and links, but the exit code is wrong. Likely a QBE codegen issue with boolean expressions and exit(0).

2. **test_strings.sb** -- Compile error: `Missing type for variable 'n'` when assigning `let n = int_to_str(42)`. Type inference does not propagate the return type of `int_to_str()` to the variable. This is a type inference gap.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical Functionality] Harness updated to collect failures instead of aborting**
- **Found during:** Task 2 execution
- **Issue:** The initial `compile_and_run_qbe_checked()` used `assert!()` which panics, causing `test_qbe_execution_tests` to abort on the first failing test program. The plan explicitly states "individual test programs may fail -- that is expected data for the gap inventory."
- **Fix:** Changed `compile_and_run_qbe_checked()` to return `Result<(), String>` with `Err` on failures. Updated `test_qbe_execution_tests()` to collect and print all results without aborting, so all gap data is visible in a single test run.
- **Files modified:** src/tests/test_examples.rs
- **Commits:** 33cee64

## Decisions Made

1. `compile_and_run_qbe_checked` uses `Result<(), String>` not `Result<(), Error>` -- assertion failures and io errors both become descriptive strings, enabling the harness to collect and report all failures rather than crashing on the first one.

2. `test_qbe_execution_tests` always returns `Ok(())` -- individual test program failures are printed as gap data, not treated as harness failures. This was the correct interpretation of the plan's "individual test programs may fail -- that is expected data for the gap inventory."

## Known Stubs

None. All test programs exercise real language features. The 2 failures are genuine gaps, not stubs.

## Self-Check: PASSED

Files created:
- tests/qbe/test_int_arithmetic.sb: FOUND
- tests/qbe/test_int_comparison.sb: FOUND
- tests/qbe/test_booleans.sb: FOUND
- tests/qbe/test_variables.sb: FOUND
- tests/qbe/test_if_else.sb: FOUND
- tests/qbe/test_while_loops.sb: FOUND
- tests/qbe/test_functions.sb: FOUND
- tests/qbe/test_strings.sb: FOUND
- src/tests/test_examples.rs: modified (FOUND)

Commits:
- 2da2391: feat(01-01): add QBE execution test harness with checked assertions
- 33cee64: feat(01-01): add 7 self-checking QBE test programs and update harness
