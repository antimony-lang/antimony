---
id: SEED-002
status: dormant
planted: 2026-03-23
planted_during: v1.0 milestone — Phase 1 QBE Stabilization and Audit
trigger_when: Before the bootstrap milestone begins
scope: Medium
---

# SEED-002: Complete the language specification to tighten the envisioned language contract

## Why This Matters

The language contract must be unambiguous before the compiler is bootstrapped. A self-hosted compiler written in Antimony will make implementation decisions for every edge case — if the spec is silent or vague, those decisions will be inconsistent or wrong. Completing the spec before the bootstrap milestone ensures the self-hosted compiler has a precise target to implement.

## When to Surface

**Trigger:** Before the bootstrap milestone begins

This seed should be presented during `/gsd:new-milestone` when the milestone scope matches any of these conditions:
- A new milestone targets bootstrapping the compiler (writing the compiler in Antimony)
- Language design is being finalized or frozen
- A new milestone adds significant language features that need speccing first

## Scope Estimate

**Medium** — A phase or two, needs planning. Involves completing formal grammar for all language constructs, resolving open TODOs in the spec, and verifying the spec against the actual parser/generator behaviour.

## Breadcrumbs

- `docs/developers/specification.md` — Existing partial spec; marked "work in progress"; TODOs for floating-point literals, rune literals, string `'` vs `"` disambiguation, byte values
- `docs/concepts/` — User-facing concept docs (datatypes, functions, variables, control-flow, structured-data, comments) that should align with formal spec
- `.planning/STATE.md` — Blockers: pointer type syntax is unresolved (affects Phase 2); enum/tagged-struct decision must be resolved before Phase 4 — both are spec gaps
- `src/lexer/` — Source of truth for current token definitions; spec should match
- `src/parser/parser.rs` — Source of truth for current grammar; spec should match

## Notes

The spec currently covers: character classes, identifiers, keywords, operators, integer literals. Still missing: floating-point literals, rune literals, formal string grammar, type system, expressions, statements, declarations, modules, and the full grammar for all constructs. The pointer and enum/tagged-struct decisions flagged in STATE.md are the highest-priority spec gaps relative to the current roadmap.
