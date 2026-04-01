# Phase 3: Standard Library - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-01
**Phase:** 03-standard-library
**Areas discussed:** Enum strategy, Dynamic array backing, String builder design, `any` type completeness

---

## Enum Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Tagged-struct convention | Document the pattern, write test programs with Token/Statement/Expression structs | |
| First-class enums | Implement enum syntax with associated data in the compiler before moving to stdlib | ✓ |

**User's choice:** First-class enums

Follow-up: Named fields vs positional:

| Option | Description | Selected |
|--------|-------------|----------|
| Named fields only | `enum Foo { Bar { x: int, y: str } }` | ✓ (first) |
| Positional payloads | `enum Foo { Bar(int, str) }` | deferred |
| Both | Some variants named, some positional | |

**User's choice:** Named fields first, positional deferred

Follow-up: Destructuring:

| Option | Description | Selected |
|--------|-------------|----------|
| Extend match statement | `match token { case Ident { name } => ... }` | ✓ |
| New syntax | Separate from match | |
| You decide | Reuse match | |

**User's choice:** Extend the existing match statement

---

## Dynamic Array Backing

| Option | Description | Selected |
|--------|-------------|----------|
| C helper functions | `_vec_new/_vec_push/_vec_get/_vec_len` in builtin_qbe.c | |
| Typed implementations | IntArray / StrArray structs in Antimony | |
| Pure Antimony struct + malloc | Depends on fixing `any` first | |

**User's choice:** "I want this to be fully abstracted from the user" — `let foo = []; foo.push(1); println(foo)` → `[1]`

Follow-up: Fixed vs dynamic array unification:

| Option | Description | Selected |
|--------|-------------|----------|
| All arrays dynamic | `[]` and `[1,2,3]` both growable, no fixed-size type | ✓ |
| Two types | `[]` dynamic, `[1,2,3]` fixed-size | |
| You decide | Whatever is cleanest | |

**User's choice:** All arrays become dynamic

Follow-up: Element type inference:

| Option | Description | Selected |
|--------|-------------|----------|
| Inferred from first push | `let foo = []; foo.push(1)` → `int[]` | ✓ |
| Always `any` | Python-style mixed-type lists | |
| Explicit annotation required | `let foo: int[] = []` | |

**User's choice:** Inferred from first push

---

## String Builder Design

| Option | Description | Selected |
|--------|-------------|----------|
| Pre-allocated buffer | StringBuilder struct with fixed-cap malloc'd buffer, write() appends in-place | ✓ |
| Array-of-strings join | Collect str[] parts, join at end | |
| Skip for now | Use str + str, optimize if bootstrap hits perf issues | |

**Notes:** User asked "what's more intuitive for a modern language?" — answered with overview of Rust/Java/Swift/Python approaches. User chose the builder pattern (most common in modern OO languages).

**User's choice:** `StringBuilder` with `append()` / `to_str()`

---

## `any` Type Completeness

| Option | Description | Selected |
|--------|-------------|----------|
| No — typed stdlib | Implement typed stdlib, skip `any` improvements | |
| Yes — fix any | Fix `any` enough for generic collections before building stdlib | ✓ |
| Defer | Use typed implementations now, revisit if bootstrap hits a gap | |

**User's choice:** Fix `any` enough to support generic collections

---

## Claude's Discretion

- QBE memory layout for enum variants
- Internal growth strategy for dynamic arrays
- Exact C helper split for dynamic arrays vs pure Antimony
- Order of implementation within the phase

## Deferred Ideas

- Positional enum variants (`Ident(str)`) — after named fields work
- `args()` → `str[]` — after dynamic arrays validated
- `io` stdlib module — Phase 3 or later
- `free` / memory management — out of scope for bootstrap
