---
phase: quick
plan: 01
subsystem: compiler-backends
tags: [cleanup, backends, ci]
dependency_graph:
  requires: []
  provides: [simplified-target-enum, no-llvm-dependency]
  affects: [src/generator, src/builder, ci-pipeline]
tech_stack:
  removed: [inkwell, llvm-sys, x86-backend, llvm-backend]
  patterns: [three-backend-architecture]
key_files:
  modified:
    - src/generator/mod.rs
    - src/builder/mod.rs
    - src/main.rs
    - Cargo.toml
    - .github/workflows/ci.yml
  deleted:
    - src/generator/x86.rs
    - src/generator/llvm.rs
decisions:
  - Only C, JS, and QBE backends remain as compilation targets
metrics:
  duration: 2m
  completed: 2026-03-23
---

# Quick Task 260323-cfg: Deprecate x86 and LLVM Backends Summary

Removed x86 and LLVM backends entirely, leaving only C, JS, and QBE as compilation targets -- eliminates LLVM build dependency from CI and simplifies the Target enum.

## Task Results

| Task | Name | Commit | Status |
|------|------|--------|--------|
| 1 | Remove x86 and LLVM generator code and all references | 81fc9b6 | Done |
| 2 | Remove LLVM from CI and run tests | 0cc16b6 | Done |

## Changes Made

### Task 1: Remove x86 and LLVM generator code

- Deleted `src/generator/x86.rs` (x86 assembly backend) and `src/generator/llvm.rs` (LLVM backend)
- Removed `Llvm` and `X86` variants from `Target` enum in `src/generator/mod.rs`
- Removed corresponding match arms in `from_extension()` and `FromStr` impl
- Removed `Target::Llvm` and `Target::X86` match arms from `src/builder/mod.rs` generate method
- Updated CLI help string in `src/main.rs` from "c, js, llvm, qbe, x86" to "c, js, qbe"
- Removed `[features]` section (`llvm = ["inkwell"]`) and `inkwell` dependency from `Cargo.toml`

### Task 2: Remove LLVM from CI

- Removed all four "Install LLVM and Clang" step blocks from CI jobs (check, test, fmt, clippy)
- Full test suite passes: 179 tests, 0 failures

## Verification

- `cargo check` succeeds
- `cargo test` passes all 179 tests
- No references to x86, X86, llvm, Llvm, LLVM, or inkwell in `src/`
- No references to llvm or LLVM in `.github/workflows/ci.yml`
- No inkwell reference in `Cargo.toml`
- Files `src/generator/x86.rs` and `src/generator/llvm.rs` confirmed deleted

## Deviations from Plan

None -- plan executed exactly as written.

## Known Stubs

None.
