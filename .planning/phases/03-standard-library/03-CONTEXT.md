# Phase 3: Standard Library - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver the data structures needed to write a compiler — dynamic arrays,
associative maps, and a string builder — plus implement first-class enums
(named-field variants, match destructuring). This phase also fixes the `any`
type enough to support generic collections. It does NOT write the self-hosted
compiler (Phase 4-5) — it ensures the language is capable enough for that work.

</domain>

<decisions>
## Implementation Decisions

### Enums
- **D-01:** First-class enums with named-field variants are implemented in Phase 3.
  Syntax: `enum Token { Ident { name: str }, Plus, Int { value: int } }`.
- **D-02:** Positional/tuple-style variants (e.g., `Ident(str)`) are deferred —
  named fields are implemented first.
- **D-03:** Enum values are destructured via the existing `match` statement,
  extended with pattern binding for named fields:
  `match token { case Ident { name } => println(name) }`.

### Dynamic Arrays
- **D-04:** All arrays become dynamic going forward. Both `[]` and `[1, 2, 3]`
  create growable arrays — there is no longer a fixed-size array type.
- **D-05:** Element type is inferred from usage (first `push()` call or context).
  `let foo = []; foo.push(1)` makes `foo` an `int[]`.
- **D-06:** `push()` is a built-in method on all arrays. `println(arr)` prints
  `[1, 2, 3]` style output.

### `any` Type
- **D-07:** Fix `any` enough to support generic collections before building stdlib.
  The goal: arrays and maps can hold values of any type, type-checked at
  compile time where possible, stored as 8-byte words under the hood.

### Associative Map
- **D-08:** Implement a generic `Map` type (str keys → any values) backed by
  parallel arrays with linear search (acceptable for bootstrap compiler use).
  API: `map.set(key, val)`, `map.get(key)`, `map.has(key)`.

### String Builder
- **D-09:** `StringBuilder` struct with `append(str)` and `to_str(): str` methods.
  Backed by a pre-allocated buffer that grows as needed (malloc + realloc pattern).
  This is the idiomatic output-building primitive for the bootstrap compiler.

### Inherited from Prior Phases
- Test programs are self-checking: print PASS/FAIL, exit 0/1 (Phase 1)
- One `.sb` test file per feature in `tests/qbe/`
- New builtins follow existing registration pattern in `infer_builtin()` (Phase 2)

### Claude's Discretion
- QBE memory layout for enum variants (tagged struct, heap-tagged pointer, etc.)
- Internal growth strategy for dynamic arrays (doubling, etc.)
- Exact C helper split for dynamic arrays vs pure Antimony implementation
- Order of implementation within the phase

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Type System and Parser
- `src/ast/hast.rs` — High-level AST; enum declaration and match variants go here
- `src/ast/last.rs` — Low-level AST; lowered enum representation
- `src/ast/transform.rs` — HAST → LAST lowering; enum desugaring goes here
- `src/parser/parser.rs` — Parser; enum declaration parsing, match pattern binding
- `src/parser/infer.rs` — Type inference; enum types, any type fixups, array element type inference

### QBE Codegen
- `src/generator/qbe.rs` — QBE backend; enum codegen, dynamic array codegen,
  StringBuilder codegen (~1753 lines)
- `builtin/builtin_qbe.c` — C helpers; dynamic array grow/push helpers,
  StringBuilder buffer management go here
- `src/generator/qbe.rs` RUNTIME_PREAMBLE — Inline QBE IL functions; new
  array/map/sb wrappers follow this pattern

### Stdlib
- `lib/array.sb` — Existing fixed-size array stdlib; will be updated/replaced for
  dynamic array API
- `lib/string.sb` — Existing string stdlib; StringBuilder may go here or new file

### Test Infrastructure
- `src/tests/test_examples.rs` — Integration test harness; new QBE execution tests
  follow patterns here
- `tests/qbe/` — Self-checking test programs; Phase 3 adds test_enums.sb,
  test_dynarray.sb, test_map.sb, test_stringbuilder.sb

### Gap Inventory
- `.planning/phases/01-qbe-stabilization-and-audit/QBE-GAPS.md` — Phase 3 gaps:
  `any` type (PARTIAL), dynamic arrays, associative arrays

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `RUNTIME_PREAMBLE` in `src/generator/qbe.rs`: existing pattern for QBE IL
  wrappers. New array/map helpers follow this.
- `builtin_qbe.c`: `_str_concat`, `_malloc` show the pattern for C helpers that
  manage heap memory.
- `infer_builtin()` in `src/parser/infer.rs`: single place to add return types for
  new builtins (array push/get, map set/get, sb append/to_str).
- Existing `match` statement support (lowered to if-else chain) — needs to be
  extended for enum variant pattern binding.
- Fixed-size array codegen in `src/generator/qbe.rs` — starting point for
  understanding how to extend to dynamic arrays.

### Established Patterns
- Arrays currently: fixed-size, stack-allocated, `Array(T, size)` in LAST.
- Match currently: lowered from HAST match → LAST if-else chain in `transform.rs`.
- New features follow: HAST definition → transform lowering → LAST representation
  → QBE codegen → builtin registration → stdlib wrapper.

### Integration Points
- Enum declaration needs a new HAST/LAST node type and a new parser rule.
- Dynamic arrays need `Array(T, size)` in LAST to become dynamic — this is a
  breaking change to the AST type; assess impact on all backends (JS, C) and
  existing tests.
- `push()` method needs special handling: it's a method on a built-in type, not
  a user-defined struct.

</code_context>

<specifics>
## Specific Ideas

- User wants `let foo = []; foo.push(1); println(foo)` → prints `[1]`. The API
  should feel like Python/modern JS, not C-style vec wrappers.
- StringBuilder is the primary output-building primitive for the bootstrap compiler
  (emitting QBE IL). The `append()` / `to_str()` pattern matches that use case.
- `any` fix goal: generic collections, not full dynamic typing. Arrays and maps
  should be usable without fighting the type system.

</specifics>

<deferred>
## Deferred Ideas

- Positional/tuple-style enum variants (`Ident(str)`) — implement after named
  fields work
- `args()` → `str[]` convenience builtin — depends on dynamic arrays being solid
  (flagged in Phase 2 context)
- `io` stdlib module — high-level file I/O wrapper (flagged in Phase 2 context)
- `free` / memory management — explicitly out of scope for bootstrap milestone

</deferred>

---

*Phase: 03-standard-library*
*Context gathered: 2026-04-01*
