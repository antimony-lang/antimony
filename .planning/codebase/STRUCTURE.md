# STRUCTURE
_Last updated: 2026-03-23_

## Summary
The codebase is a single Rust crate organized into focused modules under `src/`. Each compiler stage has its own directory. Tests live both inline (`#[cfg(test)]`) and in dedicated test files. The `lib/` and `builtin/` directories hold Antimony source files embedded at compile time.

## Directory Layout

```
/
├── src/
│   ├── main.rs                  # CLI entry point (clap dispatch)
│   ├── ast/
│   │   ├── mod.rs               # Re-exports hast/last types
│   │   ├── hast.rs              # High-level AST (HModule, HFunction, ...)
│   │   ├── last.rs              # Low-level AST (Module, Function, ...)
│   │   ├── transform.rs         # HAST → LAST lowering (AstTransformer)
│   │   ├── types.rs             # Type enum (Int, Bool, Str, Array, Struct, Any)
│   │   └── README.md
│   ├── builder/
│   │   ├── mod.rs               # Builder: module loading, import resolution, stdlib append
│   │   └── README.md
│   ├── command/
│   │   ├── mod.rs
│   │   ├── build.rs             # `sb build` handler
│   │   └── run.rs               # `sb run` handler (builds → executes via node)
│   ├── generator/
│   │   ├── mod.rs               # Generator trait, Target enum, GeneratorResult
│   │   ├── js.rs                # JavaScript backend
│   │   ├── c.rs                 # C backend
│   │   ├── qbe.rs               # QBE SSA IR backend (primary systems target)
│   │   ├── x86.rs               # x86-64 assembly backend (partial)
│   │   ├── llvm.rs              # LLVM backend (optional, incomplete)
│   │   └── tests/
│   │       ├── mod.rs
│   │       ├── js_tests.rs
│   │       ├── c_tests.rs
│   │       └── qbe_tests.rs
│   ├── lexer/
│   │   ├── mod.rs               # Lexer, TokenKind, Keyword, Value enums
│   │   ├── cursor.rs            # Character-level cursor
│   │   ├── display.rs           # Display impls for tokens
│   │   └── tests.rs             # Lexer unit tests
│   ├── parser/
│   │   ├── mod.rs               # Module re-exports
│   │   ├── parser.rs            # Parser state machine
│   │   ├── rules.rs             # Parsing rules (grammar productions)
│   │   ├── infer.rs             # Type inference pass
│   │   └── tests.rs             # Parser unit tests
│   ├── tests/
│   │   ├── mod.rs
│   │   └── test_examples.rs     # Integration tests: compile examples/ via `cargo run build`
│   └── util/
│       ├── mod.rs
│       └── string_util.rs       # highlight_position_in_file() for error reporting
├── lib/                         # Antimony standard library (embedded via rust-embed)
├── builtin/                     # Built-in function implementations (embedded)
├── examples/                    # Example .sb programs (used in integration tests)
├── docs/                        # mdBook documentation source
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml          # Pins Rust 1.93, minimal profile
├── book.toml                    # mdBook config
├── Dockerfile
└── .github/workflows/ci.yml     # CI pipeline (check, test, fmt, clippy)
```

## Module Responsibilities

| Module | Responsibility |
|---|---|
| `lexer` | Source text → token stream |
| `parser` | Token stream → HModule (high-level AST) |
| `parser/infer` | Type annotation of HModule |
| `ast/transform` | HModule → Module (low-level AST) |
| `builder` | File loading, import resolution, stdlib injection |
| `generator/*` | Module → target language string |
| `command/*` | CLI subcommand handlers |
| `util` | Error formatting helpers |

## Key File Sizes (estimated scope)
- `src/generator/qbe.rs` — largest file (>600 lines, most complex backend)
- `src/parser/parser.rs` / `rules.rs` — substantial (grammar + recovery)
- `src/ast/hast.rs` + `last.rs` — relatively small type definitions

## Gaps & Unknowns
- `lib/` and `builtin/` contents not inventoried here
- `examples/` contents not enumerated (used by integration tests)
- No workspace structure — single crate
