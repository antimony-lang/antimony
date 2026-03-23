# ARCHITECTURE
_Last updated: 2026-03-23_

## Summary
Antimony uses a classic multi-stage compiler pipeline: source → tokens → high-level AST → low-level AST → backend output. The key architectural decision is a two-level AST (HAST/LAST) that separates language-level constructs from code-generation-ready constructs. Multiple backends implement a single `Generator` trait.

## Pipeline Stages

```
Source (.sb)
    │
    ▼
Lexer (src/lexer/)
  TokenKind stream
    │
    ▼
Parser (src/parser/)
  HModule (High-level AST)
    │
    ▼
Type Inference (src/parser/infer.rs)
  Annotated HModule
    │
    ▼
AST Transformer (src/ast/transform.rs)
  Module (Low-level AST)
    │
    ▼
Generator (src/generator/{js,c,qbe,x86,llvm}.rs)
  Target output string
```

## Two-Level AST

**High-level AST (`src/ast/hast.rs`)**
- `HModule`, `HFunction`, `HStatement`, `HExpression`
- Contains: match statements, for-in loops, imports tracking (`HashSet<String>`)
- Produced by the parser; accumulated during module loading

**Low-level AST (`src/ast/last.rs`)**
- `Module`, `Function`, `Statement`, `Expression`
- Contains only constructs with direct backend analogues: if/while/for, block, declare, assign, return, binop
- No match expressions — lowered to if-else chains by the transformer
- Produced by `AstTransformer`

## Builder and Module System (`src/builder/mod.rs`)
- `Builder::build_module()` recursively loads `.sb` files following import declarations
- Circular imports prevented via `seen: Vec<String>` passed by reference through recursion
- Modules merged with `HModule::merge_with()` — all functions/structs/globals flattened into one module
- Standard library (`lib/`) automatically appended for JS and QBE targets
- Temporarily changes `env::current_dir` to resolve relative imports — restored after build

## Generator Pattern (`src/generator/mod.rs`)
```rust
pub trait Generator {
    fn generate(prog: Module) -> GeneratorResult<String>;
}
pub type GeneratorResult<T> = Result<T, String>;
```
- All backends are stateless from the caller's perspective (take owned `Module`, return `String`)
- Target selected by output file extension (`.c`, `.js`, `.ssa`, `.s`) or `--target` flag
- `QbeGenerator` is stateful internally (scoped variables, temporary counters, struct maps)

## QBE Generator Details (`src/generator/qbe.rs`)
Key state in `QbeGenerator`:
- `scopes: Vec<HashMap<String, VarInfo>>` — lexical scoping for variables
- `struct_map` — struct name → (qbe type, field metadata, size)
- `loop_labels` — stack for break/continue label generation
- `fn_signatures` / `fn_param_types` — populated in a pre-pass before codegen
- `intrinsics: HashMap<String, Intrinsic>` — inline builtins (e.g. `len` → `ArrayLen`)
- Uses `Rc<qbe::TypeDef>` to avoid lifetime issues with the qbe crate

## Type System (`src/ast/types.rs`)
- Types: `Int`, `Bool`, `Str`, `Array(Box<Type>)`, `Struct(String)`, `Any`
- `Option<Type>` used pervasively — `None` means unknown/untyped
- Type inference (`infer.rs`) propagates types through symbol tables; limited (function return inference duplicated in QBE backend)

## Entry Points
- `src/main.rs` — clap CLI dispatch → `build` or `run` subcommand
- `src/command/build.rs` — orchestrates Builder + Generator, writes output file
- `src/command/run.rs` — builds to JS, executes with Node.js

## Data Flow Notes
- Parser state: current token, peek buffer, raw source string (for error positions)
- Error propagation: `Result<T, String>` everywhere with `?` operator
- No IR optimization pass — QBE handles optimization on its side
- No separate semantic analysis pass — type constraints enforced during parsing

## Gaps & Unknowns
- x86 generator (`src/generator/x86.rs`) — completeness unclear
- LLVM generator is largely stubbed with `todo!()` calls
- No multi-file compilation caching — every build re-parses all transitive imports
