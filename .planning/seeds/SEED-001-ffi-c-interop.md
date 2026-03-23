---
id: SEED-001
status: dormant
planted: 2026-03-23
planted_during: v1.0 / Phase 1 — QBE Stabilization and Audit
trigger_when: after the compiler is self-hosting
scope: Medium
---

# SEED-001: Add FFI to call C libraries from Antimony

## Why This Matters

There is currently no way to declare or call external C functions from Antimony code. Without FFI, every subsystem must be reimplemented from scratch in Antimony — including things already solved by libc, SDL, and other C libraries. FFI would allow Antimony programs to reuse the existing C ecosystem, which is critical for building real-world programs like a Doom port without duplicating massive amounts of C infrastructure.

## When to Surface

**Trigger:** After the compiler is self-hosting

This seed should be presented during `/gsd:new-milestone` when the milestone scope matches any of these conditions:
- The bootstrap milestone has been completed (compiler written in Antimony, compiled via QBE)
- Work begins on the Doom port or other systems programs that need C interop
- Standard library development reaches a point where libc bindings would be valuable

## Scope Estimate

**Medium** — A phase or two. Needs design work for `extern` declaration syntax, parser support, type-checking of foreign signatures, and QBE IL emission for external symbol calls. Does not require a full milestone but is more than a quick task.

## Breadcrumbs

Related code and decisions found in the current codebase:

- `src/ast/hast.rs:31` — `pub imports: HashSet<String>` — module import tracking; would need extension to distinguish foreign (C) modules from Antimony modules
- `src/parser/rules.rs:785` — `parse_declare()` — existing declare statement parsing; natural anchor point for an `extern fn` declaration syntax
- `src/generator/qbe.rs` — QBE code generator; extern calls in QBE IL use `call $symbol(...)` with no function body — needs handling here
- `src/builder/mod.rs` — import resolution pipeline; foreign modules should be skipped during AST building (no source to parse)

## Notes

The most natural design is an `extern` keyword for declaring C function signatures without a body, similar to Rust's `extern "C"` blocks. QBE IL already supports calling external symbols natively, so the generator changes should be straightforward once the AST and parser are extended.
