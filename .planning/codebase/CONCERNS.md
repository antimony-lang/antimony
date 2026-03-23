# CONCERNS
_Last updated: 2026-03-23_

## Summary
The codebase has a manageable amount of technical debt, concentrated in four areas: the LLVM backend (largely unimplemented), the QBE backend's type inference duplication, the builder's environment mutation pattern, and the integration test suite's JS-only coverage. The most critical gap relative to project goals is that the QBE backend has no end-to-end correctness tests.

## Technical Debt

### Type Inference Duplication
- `src/generator/qbe.rs:71` — `infer_fn_return_type()` lives in the QBE backend but belongs in `src/parser/infer.rs`
- The TODO comment acknowledges this: "This logic belongs in `src/parser/infer.rs`, which should populate `HFunction::ret_type` so all backends get inferred return types for free"
- Risk: other backends may have inconsistent type inference behavior

### Builder Directory Mutation
- `src/builder/mod.rs:57-70` — builder temporarily changes `env::current_dir` to resolve relative imports
- Comment: `"TODO: This error could probably be handled better"` and `"TODO: This method can probably cleaned up quite a bit"`
- Risk: not thread-safe; if building is ever parallelized this pattern breaks

### Unnecessary Clones
- `src/builder/mod.rs:127` — `// TODO: We shouldn't clone here`
- `src/parser/rules.rs:131` — `// TODO: Not sure if we should clone here`
- Performance concern for large programs, not correctness

### Global Symbol Table
- `src/parser/infer.rs:23` — `// TODO: Global symbol table is passed around randomly.`
- Type inference state management is ad-hoc

### Incomplete Import Tracking
- `src/parser/rules.rs:46` — `// TODO: Populate imports`
- Import resolution may not be fully tracked in the AST

## Known Issues (TODO/FIXME)

| File | Line | Issue |
|---|---|---|
| `src/lexer/mod.rs` | 228 | `FIXME: Identical value, since it will be used twice and is not clonable later` |
| `src/lexer/mod.rs` | 425 | `FIXME: Might lead to a bug, if End of file is encountered` |
| `src/parser/infer.rs` | 150 | `// TODO: This approach only relies on the first element` (array type inference) |
| `src/parser/mod.rs` | 17 | `// TODO: Resolve this lint by renaming the module` |
| `src/generator/js.rs` | 112 | `// TODO: Prepend statements` |
| `src/generator/js.rs` | 268 | `// TODO: Can let be used instead?` (var scoping) |
| `src/util/string_util.rs` | 22 | `// TODO: do something better, code can be more than 9999 lines` (error formatting) |

## Incomplete Implementations

### LLVM Backend (`src/generator/llvm.rs`)
Multiple `todo!()` panics for common type cases:
- `Some(Type::Any) => todo!()`
- `Some(Type::Str) => todo!()`
- `Some(Type::Array(_)) => todo!()`
- `Some(Type::Struct(_)) => todo!()`
- Generic statement/expression handlers: `_ => todo!()`

The LLVM backend will panic at runtime on any non-trivial program. It is not a viable target.

### x86 Backend (`src/generator/x86.rs`)
- Marked as partial in documentation; scope of completeness unknown

## Risks Relative to Bootstrap Goal

### QBE Backend Correctness
- No end-to-end integration tests for QBE (`.sb → .ssa → binary → run`)
- Unit tests check IR string output but not execution correctness
- The bootstrap goal requires the QBE backend to correctly compile the entire Antimony compiler (itself a complex Rust-like program)
- Gap: needs integration test suite running real programs through the QBE binary

### Missing Language Features for Bootstrap
The compiler is written in Rust; bootstrapping requires Antimony to express:
- Recursive data structures (linked lists / trees for AST) — structs exist but complex nesting unclear
- String manipulation (for lexer/parser) — `Type::Str` exists but stdlib coverage unknown
- File I/O — not confirmed in standard library
- HashMap-equivalent — no built-in map type visible in the AST
- Pattern matching beyond basic if/else — match is lowered to if-else

### CI Stale Dependencies
- `actions-rs/cargo@v1` is deprecated
- `actions/checkout@v2` is two major versions behind (v4 is current)
- `KyleMayes/install-llvm-action` pins LLVM 10 (released 2020); LLVM 18+ is current

## Missing Language Features (Type System)
Confirmed from `src/ast/types.rs` — the `Type` enum only has:
`Any`, `Int`, `Str`, `Bool`, `Array(Box<Type>, Option<usize>)`, `Struct(String)`

Notable absences relevant to the bootstrap goal:
- **No float type** — rules out floating-point math
- **No pointer/reference type** — no raw memory access, no C interop at the type level
- **No function type / closures** — callbacks and higher-order functions not expressible
- **No unsigned integer / sized integers** (no u8, u32, i64, etc.) — `Int` is untyped/single-width
- **No void/unit type** — functions without return use `Option<Type> = None` implicitly

These gaps are load-bearing for bootstrapping: the Rust compiler relies heavily on typed integers, pointers, and closures.

## Gaps & Unknowns
- Standard library (`lib/`) contents not surveyed — unknown what primitives exist
- `builtin/` contents not surveyed — unknown what built-in functions are implemented
- Actual x86 backend coverage unknown
- No fuzzing or property-based testing for the parser
