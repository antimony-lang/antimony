# Phase 1: QBE Stabilization and Audit - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Establish a trusted QBE backend with execution tests and a complete gap inventory. This phase does NOT add new language features or fix gaps — it maps the terrain and ensures what exists is tested and trustworthy.

</domain>

<decisions>
## Implementation Decisions

### Execution Test Strategy
- **D-01:** Expand the integration test suite with new .sb programs that compile and run through the full QBE pipeline (`.sb` -> `.ssa` -> `.s` -> binary -> execution). Keep existing 48 IL snapshot unit tests as-is.
- **D-02:** Execution tests validate both stdout content AND exit code (most thorough).
- **D-03:** New QBE execution tests live in `tests/qbe/` directory, separate from `examples/` (which stays clean for documentation).

### Transmute Resolution
- **D-04:** Fix the lifetime issue upstream in the `qbe` crate (user owns the crate). Relax lifetime constraints or add an owned type variant so the two `unsafe { std::mem::transmute }` calls (lines 201 and 1632 of `src/generator/qbe.rs`) can be removed. No fallback needed — user will merge the fix.
- **D-05:** After the crate fix is published, update `Cargo.toml` to the new version and remove both transmutes from the generator.

### Gap Inventory
- **D-06:** Gap inventory is a feature checklist with severity: table listing every language feature, QBE status (pass/fail/partial), and severity (blocks bootstrap / needed later / nice-to-have).
- **D-07:** Gap inventory lives at `.planning/phases/01-qbe-stabilization-and-audit/QBE-GAPS.md` as a phase artifact.
- **D-08:** Each gap is cross-referenced to the roadmap phase that resolves it (e.g., "string indexing -> Phase 2: Runtime Primitives"). Makes the inventory a planning tool, not just a list.

### Test Program Coverage
- **D-09:** Systematic language feature sweep — write a test program for every language feature (each type, control flow construct, expression kind, structs, arrays, methods, etc.) to get the full picture for the gap inventory.
- **D-10:** One .sb test file per feature (e.g., `test_structs.sb`, `test_while_loops.sb`, `test_string_ops.sb`). Easy to isolate failures.
- **D-11:** Test programs are self-checking: each .sb does its own assertions, prints 'PASS' or 'FAIL: reason', exits 0 or 1. The test harness just checks exit code and scans for FAIL.

### Claude's Discretion
- Order of language features to test (can be derived from the AST/parser feature set)
- Internal structure of the test harness (how `test_examples_qbe` pattern is extended)
- How to enumerate language features systematically (walk the AST enum variants, parser capabilities, etc.)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### QBE Generator
- `src/generator/qbe.rs` — The 1,753-line QBE backend; contains the two transmutes to fix and all codegen logic
- `src/generator/tests/qbe_tests.rs` — 48 IL snapshot tests (1,898 lines); the existing unit test baseline

### Test Infrastructure
- `src/tests/test_examples.rs` — Integration test harness; `test_examples_qbe()` and `test_testcases_qbe()` are the patterns to extend
- `examples/` — 8 existing example programs compiled through QBE in CI
- `tests/` — Existing test case directory

### Builtins and Dependencies
- `builtin/builtin_qbe.c` — 7 C builtins linked at compile time (printf, exit, int_to_str, str_concat, strlen, parse_int, read_line)
- `Cargo.toml` — QBE crate dependency (currently 3.0.0; will be updated after upstream fix)

### AST (for systematic feature enumeration)
- `src/ast/hast.rs` — High-level AST; defines all language features the compiler supports
- `src/ast/last.rs` — Low-level AST; defines what the QBE generator actually receives after transform

### CI
- `.github/workflows/ci.yml` — CI pipeline; downloads QBE 1.2, compiles from source, runs full test suite

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `test_examples_qbe()` in `src/tests/test_examples.rs`: Existing pattern for compile+link+execute QBE tests. New test harness should follow this pattern.
- `normalize_qbe()` helper in unit tests: Whitespace-insensitive IL comparison for snapshot tests.
- `builtin/builtin_qbe.c`: C runtime builtins already support printf and exit — enough for self-checking test programs.

### Established Patterns
- Integration tests compile `.sb` -> `.ssa` via the compiler, then shell out to `qbe` and `gcc` to produce binaries.
- Unit tests build AST nodes using helper functions (`module()`, `func()`, `block()`, `var()`), generate IL, and compare text.
- The generator implements the `Generator` trait with a single `generate(Module) -> String` method.

### Integration Points
- New `tests/qbe/` directory will need a Rust test function (likely in `src/tests/test_examples.rs` or a new test module) that discovers and runs all `.sb` files in that directory.
- The gap inventory (QBE-GAPS.md) will be produced by running the systematic test sweep and recording results.
- The `qbe` crate fix is a separate upstream task that produces a new crate version.

</code_context>

<specifics>
## Specific Ideas

- User owns the `qbe` crate — transmute fix is a direct upstream change, not a PR to a third party.
- Self-checking test pattern: each `.sb` prints PASS/FAIL and returns exit code. Keeps the Rust harness minimal.
- Gap inventory cross-references roadmap phases — serves double duty as audit artifact AND planning input for Phases 2-5.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-qbe-stabilization-and-audit*
*Context gathered: 2026-03-23*
