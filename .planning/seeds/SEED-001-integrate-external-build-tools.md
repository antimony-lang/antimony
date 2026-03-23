---
id: SEED-001
status: dormant
planted: 2026-03-23
planted_during: v1.0 / Phase 1 (QBE Stabilization and Audit)
trigger_when: When compiler performance or reliability becomes a priority
scope: Small
---

# SEED-001: Replace shell-out to qbe/gcc with cleaner integration

## Why This Matters

Shelling out to `qbe` and `gcc` is fragile: version mismatches, PATH issues, and
hard-to-test behaviour. If the tool isn't on PATH or is the wrong version, the
compiler silently fails or produces confusing errors with no actionable message.
Replacing or wrapping these invocations with a more principled integration would
make the compiler more reliable and easier to test.

## When to Surface

**Trigger:** When compiler performance or reliability becomes a priority — e.g.,
when stabilizing the QBE pipeline for bootstrap work, or when reliability issues
with the external toolchain become a recurring pain point.

This seed should be presented during `/gsd:new-milestone` when the milestone
scope matches any of these conditions:
- Milestone focuses on compiler reliability, robustness, or test coverage
- Milestone involves distributing or packaging the compiler (no assumed PATH)
- Bootstrap milestone begins (self-hosting requires a highly reliable pipeline)

## Scope Estimate

**Small** — A few hours. The shelling-out is already isolated in
`src/command/run.rs`. Possible approaches:
- Wrap invocations with better error messages and version checks
- Use the `qbe` Rust crate's API more directly instead of shelling out to the CLI
- Or bundle/vendor the `qbe` binary for hermetic builds

## Breadcrumbs

Related code in the current codebase:

- `src/command/run.rs:95-103` — shells out to `qbe -o <asm> <ssa>` then `gcc` to link
- `src/tests/test_examples.rs:98-117` — integration tests also shell out to `qbe` and `gcc` directly
- `Cargo.toml` — already depends on the `qbe 3.0.0` crate (IR generation); unclear if it exposes a compilation API

## Notes

The `qbe` Rust crate is already a dependency for IR generation. It may be worth
checking whether it also exposes a way to invoke the QBE compiler in-process
rather than via a subprocess. If not, at minimum the shell-out should validate
the tool version and emit actionable errors on failure.
