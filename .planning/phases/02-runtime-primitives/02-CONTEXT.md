# Phase 2: Runtime Primitives - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Make Antimony programs compiled via QBE capable of string operations, file I/O, CLI argument access, and heap allocation. Also fixes the 5 bootstrap-blocking bugs identified in Phase 1's gap audit. This phase does NOT build the stdlib abstraction layer (Phase 3) — it establishes the low-level primitives that stdlib will wrap.

</domain>

<decisions>
## Implementation Decisions

### File I/O
- **D-01:** File I/O implemented as thin QBE IL preamble wrappers around C stdlib (`fopen`, `fgets`/`fread`, `fwrite`, `fclose`). Same pattern as existing `_printf`, `_exit`, `_strlen` in `RUNTIME_PREAMBLE`. Exposed as low-level Antimony builtins. An `io` Antimony stdlib wrapper is deferred to Phase 3 or later.

### Heap Allocation
- **D-02:** `malloc(size: int)` → pointer exposed as a builtin. No `free`. This is an escape hatch — the intended abstraction for most Antimony code is Phase 3 stdlib (dynamic arrays, string builder), which will call `malloc` internally. Exposing it now unblocks the bootstrap if Phase 3 stdlib isn't complete yet.

### Type Inference Fixes (Bootstrap-Blocking Bugs)
- **D-03:** Fix type inference broadly, not minimally:
  - All builtins get return types at registration (not just `len`) — fixes `let n = int_to_str(42)` and all Phase 2 new builtins
  - Method calls consult struct method return types properly — fixes `let v = obj.method()`
  - `self.field = expr` parser bug fixed as a targeted patch (not a systemic change)
  - Goal: prevent the same class of inference failure from surfacing again in Phase 4/5 bootstrap work

### CLI Arguments
- **D-04:** Two builtins: `argc()` → int and `argv(i: int)` → str. Direct mapping to C's `main(int argc, char** argv)`. Safe — avoids depending on string-array codegen. A future `args()` → str[] convenience builtin is noted as a deferred improvement.

### Inherited from Phase 1
- Test programs are self-checking: print PASS/FAIL, exit 0/1
- One .sb test file per feature in `tests/qbe/`
- New builtins follow the same registration pattern as existing ones

### Claude's Discretion
- Exact C stdlib functions chosen for file I/O wrappers (fopen/fgets vs fopen/fread — whichever handles text files cleanly)
- Internal structure of builtin registration for return types
- Order of bug fixes vs new feature implementation within the phase

</decisions>

<specifics>
## Specific Ideas

- "QBE already has C interop — use that and maybe add an io stdlib wrapper later" — file I/O should be thin QBE preamble wrappers, not a new C file
- Heap allocation should be abstracted away from users by default (Phase 3 stdlib), but accessible if needed — `malloc` is the escape hatch, not the primary API
- `args()` → str[] is the ergonomic goal eventually, but `argc()`/`argv(i)` is the safe foundation

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Runtime and Builtins
- `src/generator/qbe.rs` — QBE backend; `RUNTIME_PREAMBLE` (line ~35) defines inline QBE IL functions; new file I/O and CLI arg wrappers go here
- `builtin/builtin_qbe.c` — C builtins linked at compile time; `malloc` wrapper goes here if not expressible in QBE IL directly

### Type Inference
- `src/parser/infer.rs` — Type inference pass; `infer_builtin()` (~line 165) is where builtin return types are registered; `infer_function_call()` and `infer_expression()` are the entry points to fix
- `src/ast/hast.rs` — HAST definitions; method call expression variant to check for inference gap

### Parser (self.field assignment bug)
- `src/parser/parser.rs` — Parser; find the assignment statement parsing path that fails for `self.field = expr`

### Gap Inventory (Phase 2 work items)
- `.planning/phases/01-qbe-stabilization-and-audit/QBE-GAPS.md` — The 5 bootstrap-blocking gaps this phase must close (Bool codegen, And/Or operators, Str type inference, method return type inference, self field assignment)

### Test Infrastructure
- `src/tests/test_examples.rs` — Integration test harness; new QBE execution tests follow patterns here
- `tests/qbe/` — Existing self-checking test programs; new test programs go here

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `RUNTIME_PREAMBLE` in `src/generator/qbe.rs`: already defines QBE IL wrappers for `_printf`, `_exit`, `_strlen`, `_parse_int`. New file I/O wrappers (`_fopen`, `_fgets`, `_fwrite`, `_fclose`) follow the same pattern.
- `builtin_qbe.c`: `_str_concat`, `_int_to_str`, `_read_line` show the pattern for C helpers that need `malloc`. `malloc` itself can go here.
- `infer_builtin()` in `src/parser/infer.rs`: the single place to add return types for all builtins. Currently only knows `len → int`.

### Established Patterns
- Builtins are registered as QBE IL function definitions in `RUNTIME_PREAMBLE`, then called by name from Antimony code.
- C functions that need `malloc` or complex logic live in `builtin_qbe.c`; simple syscall/libc wrappers live inline in `RUNTIME_PREAMBLE`.
- Type inference is stateless — `infer_builtin` is a pure match on name → return type.

### Integration Points
- Adding a new builtin requires: (1) QBE IL declaration in `RUNTIME_PREAMBLE` or implementation in `builtin_qbe.c`, (2) return type entry in `infer_builtin()`, (3) a self-checking test in `tests/qbe/`.
- The parser's assignment path needs to recognize `self.field` (a FieldAccess expression) as a valid LHS — check how `Assign` statements are parsed vs how field access is parsed.

</code_context>

<deferred>
## Deferred Ideas

- `args()` → str[] convenience builtin — cleaner ergonomics than `argc()`/`argv(i)`, but depends on string-array codegen being solid. Add after Phase 3 stdlib validates array-of-strings handling.
- `io` Antimony stdlib module — wraps `_fopen`/`_fgets`/`_fwrite`/`_fclose` into a higher-level interface. Deferred to Phase 3 or later.
- `free` / memory management — explicitly out of scope; bootstrap compilers are batch programs that can leak.

</deferred>

---

*Phase: 02-runtime-primitives*
*Context gathered: 2026-04-01*
