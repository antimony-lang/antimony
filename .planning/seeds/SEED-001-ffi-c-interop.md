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

**Medium** — A phase or two, though likely closer to one focused phase. Needs design work for `extern` declaration syntax, parser support, type-checking of foreign signatures, and a trivial generator change. Because QBE handles the interop layer natively (external symbol linkage works out of the box), the complexity is lower than a typical FFI implementation — the estimate of "a phase or two" may be conservative; a single focused phase is plausible.

## Breadcrumbs

Related code and decisions found in the current codebase:

- `src/ast/hast.rs:31` — `pub imports: HashSet<String>` — module import tracking; would need extension to distinguish foreign (C) modules from Antimony modules
- `src/parser/rules.rs:785` — `parse_declare()` — existing declare statement parsing; natural anchor point for an `extern fn` declaration syntax
- `src/generator/qbe.rs` — QBE code generator; extern calls in QBE IL use `call $symbol(...)` with no function body — needs handling here
- `src/builder/mod.rs` — import resolution pipeline; foreign modules should be skipped during AST building (no source to parse)

## Notes

**QBE IL handles external symbol linkage natively.** QBE's `call $symbol(...)` syntax works for any external C symbol without special configuration — no ABI shim or linker script is needed. This is the hard part of FFI and QBE already solves it. The generator change is therefore minimal: emit `call $fn_name(...)` for `extern` calls the same way it does for internal calls, without emitting a function body for the extern-declared function.

**Antimony's work is purely in the frontend.** The remaining implementation is confined to the parser/AST layer:

1. `extern` keyword added to the lexer.
2. `extern fn name(args) -> ret` declaration syntax in the parser — natural anchor: `parse_declare()` in `src/parser/rules.rs:785`.
3. A new AST variant (e.g., `HStatement::ExternFn` or a flag on the existing function node) to represent a bodyless foreign declaration.
4. The transformer must pass extern declarations through to LAST without requiring a body.
5. The QBE generator skips emitting a function body for extern-flagged functions.
6. The builder must skip trying to parse a source file for `extern` imports (no Antimony source exists for a C library).

The most natural design is an `extern` keyword for declaring C function signatures without a body, similar to Rust's `extern "C"` blocks. Because QBE already solves the hard part (symbol linkage), the bulk of the effort lands in steps 1–4 above rather than in code generation.
