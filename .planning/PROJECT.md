# Antimony

## What This Is

Antimony is a personal-project compiled programming language with a multi-backend architecture (JS, C, QBE, LLVM, x86). The immediate goal is to mature the QBE backend until two milestones are reached: a bootstrapped compiler (the Antimony compiler written in Antimony and compiled via QBE) and a Doom port written in Antimony.

## Core Value

The QBE backend must become capable enough that real systems programs — including the compiler itself — can be written in Antimony and compiled correctly.

## Requirements

### Validated

- ✓ Multi-backend code generation (JS, C, QBE, LLVM, x86) — existing
- ✓ Lexer, parser, type inference pipeline — existing
- ✓ Two-level AST (HAST → LAST) with desugaring — existing
- ✓ Module system with import resolution and circular import detection — existing
- ✓ Standard library (JS and QBE targets) — existing
- ✓ Basic QBE codegen for functions, control flow, arithmetic — existing
- ✓ Integration tests: examples compiled and executed end-to-end — existing

### Active

- [ ] Systematic gap audit: identify all language features not yet working in QBE codegen
- [ ] File I/O primitives in QBE (read source files, write output)
- [ ] Pointer types and pointer arithmetic in QBE
- [ ] Dynamic memory allocation (malloc/free) in QBE
- [ ] Enums / sum types (needed for token/AST node representation)
- [ ] Mutable strings and string operations in QBE
- [ ] All language features required to write the Antimony compiler in Antimony
- [ ] Bootstrapped Antimony compiler — full rewrite of the compiler in Antimony, compiled via QBE
- [ ] Doom milestone — scope TBD after bootstrap is reached

### Out of Scope

- JS backend improvements — JS backend served its purpose; QBE is the focus
- LLVM and x86 backends — not the current focus; effort is on QBE maturity
- Language redesign or syntax changes — this milestone is about backend capability, not language evolution
- Doom milestone scope — intentionally deferred until bootstrap proves what the language can handle

## Context

- The compiler was originally JS-only, which provided a rich runtime for free. Switching to QBE exposed gaps in what the language and runtime provide natively — many of these gaps are still unknown.
- QBE is a lightweight compiler backend (used as the primary "systems" target). The compiled pipeline is: `.sb → (antimony) → .ssa → (qbe) → .s → (gcc) → binary`.
- The codebase is a Rust compiler with a clean layered architecture: Lexer → Parser → Type Inference → HAST → LAST → Generator per target.
- Self-hosting requires the language to express a full compiler: lexer, parser, data structures, symbol tables, file I/O, string handling. This is a high bar.
- The approach is **systematic audit first** — map what works in QBE now, identify gaps relative to what a self-hosting compiler needs, then close gaps in priority order.
- Doom scope is intentionally left open until bootstrap reveals what the language can handle.

## Constraints

- **Tech Stack**: QBE as the primary backend — all systems-level work must target QBE
- **Bootstrap**: The bootstrapped compiler must be a full rewrite (not a subset), compiled via QBE
- **Personal project**: No team, no deadlines — prioritize learning and correctness over velocity

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Bootstrap before Doom | Doom scope is unknown; bootstrap forces exactly the language features needed for real systems programs | — Pending |
| Systematic gap audit first | JS→QBE transition left unknown gaps; auditing prevents building on broken foundations | — Pending |
| Full compiler rewrite (not subset) | Partial bootstrap doesn't prove the language is capable; full rewrite is the real milestone | — Pending |
| Doom scope deferred | Bootstrap will reveal what the language can and can't do — scope Doom from that reality | — Pending |

---

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-23 after initialization*
