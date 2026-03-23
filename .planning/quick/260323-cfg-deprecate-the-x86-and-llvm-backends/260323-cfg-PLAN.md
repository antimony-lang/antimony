---
phase: quick
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/generator/mod.rs
  - src/generator/x86.rs
  - src/generator/llvm.rs
  - src/builder/mod.rs
  - src/main.rs
  - Cargo.toml
  - .github/workflows/ci.yml
autonomous: true
requirements: []
must_haves:
  truths:
    - "x86 and LLVM backends no longer exist in the codebase"
    - "Compiler builds and all existing tests pass without x86/LLVM code"
    - "CLI help no longer mentions llvm or x86 as target options"
    - "CI no longer installs LLVM"
  artifacts:
    - path: "src/generator/mod.rs"
      provides: "Target enum with only C, JS, Qbe variants"
    - path: "Cargo.toml"
      provides: "No inkwell/llvm dependencies or llvm feature flag"
    - path: ".github/workflows/ci.yml"
      provides: "CI without LLVM install steps"
  key_links:
    - from: "src/generator/mod.rs"
      to: "src/builder/mod.rs"
      via: "Target enum match arms"
      pattern: "Target::(C|JS|Qbe)"
---

<objective>
Remove the x86 and LLVM backends entirely. These backends are unused and unmaintained -- the project focus is exclusively on QBE.

Purpose: Clean up dead code, remove the LLVM build dependency (which slows CI and complicates setup), and simplify the Target enum.
Output: A compiler with only C, JS, and QBE backends.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@CLAUDE.md
@src/generator/mod.rs
@src/builder/mod.rs
@src/main.rs
@Cargo.toml
@.github/workflows/ci.yml
</context>

<tasks>

<task type="auto">
  <name>Task 1: Remove x86 and LLVM generator code and all references</name>
  <files>src/generator/x86.rs, src/generator/llvm.rs, src/generator/mod.rs, src/builder/mod.rs, src/main.rs, Cargo.toml</files>
  <action>
1. Delete `src/generator/x86.rs` and `src/generator/llvm.rs` entirely.

2. In `src/generator/mod.rs`:
   - Remove `#[cfg(feature = "llvm")] pub mod llvm;`
   - Remove `pub mod x86;`
   - Remove `Llvm` and `X86` variants from the `Target` enum (keep C, JS, Qbe)
   - Remove `"s" => Some(Self::X86)` from `from_extension()`
   - Remove `"llvm" => Ok(Target::Llvm)` and `"x86" => Ok(Target::X86)` from `FromStr` impl

3. In `src/builder/mod.rs`:
   - Remove the `Target::Llvm => { ... }` match arm (lines 142-148)
   - Remove the `Target::X86 => ...` match arm (line 150)

4. In `src/main.rs`:
   - Update the target help string from `"Target language. Options: c, js, llvm, qbe, x86"` to `"Target language. Options: c, js, qbe"`

5. In `Cargo.toml`:
   - Remove the `[features]` section entirely (`llvm = ["inkwell"]`)
   - Remove the `inkwell` dependency line
  </action>
  <verify>
    <automated>cd /Users/garrit/conductor/workspaces/antimony/lincoln && cargo check 2>&1 | tail -5</automated>
  </verify>
  <done>All x86 and LLVM code is removed. `cargo check` succeeds with no errors. Target enum has only C, JS, Qbe.</done>
</task>

<task type="auto">
  <name>Task 2: Remove LLVM from CI and run tests</name>
  <files>.github/workflows/ci.yml</files>
  <action>
1. In `.github/workflows/ci.yml`, remove ALL four "Install LLVM and Clang" step blocks (the `- name: Install LLVM and Clang` through `directory: ...` lines) from the check, test, fmt, and clippy jobs. These are no longer needed since the LLVM feature and dependency are gone.

2. Run the full test suite to confirm nothing is broken.
  </action>
  <verify>
    <automated>cd /Users/garrit/conductor/workspaces/antimony/lincoln && cargo test 2>&1 | tail -20</automated>
  </verify>
  <done>CI config no longer references LLVM. All existing tests pass (C, JS, QBE test suites unaffected).</done>
</task>

</tasks>

<verification>
- `cargo check` succeeds
- `cargo test` passes all tests
- `grep -r "x86\|X86\|llvm\|Llvm\|LLVM\|inkwell" src/` returns no results
- `grep "llvm\|LLVM" .github/workflows/ci.yml` returns no results
- `grep "inkwell" Cargo.toml` returns no results
- Files `src/generator/x86.rs` and `src/generator/llvm.rs` do not exist
</verification>

<success_criteria>
- The compiler builds cleanly with only C, JS, and QBE backends
- All existing tests pass
- No references to x86 or LLVM remain in source code, Cargo.toml, or CI config
- CLI `--help` shows only c, js, qbe as target options
</success_criteria>

<output>
After completion, create `.planning/quick/260323-cfg-deprecate-the-x86-and-llvm-backends/260323-cfg-SUMMARY.md`
</output>
