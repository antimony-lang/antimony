# STACK
_Last updated: 2026-03-23_

## Summary
Antimony is a compiled systems programming language implemented entirely in Rust 1.93. It compiles `.sb` source files to five backends: JavaScript, C, QBE SSA IR, x86-64 assembly, and LLVM IR. The QBE backend is the primary systems-level target; LLVM is optional and gated behind a feature flag.

## Implementation Language
- **Rust 1.93** — all compiler logic (lexer, parser, AST, backends, CLI)
- Managed via `rust-toolchain.toml` (minimal profile)
- Cargo edition 2018

## Compiler Binary
- Crate name: `antimony-lang`
- Binary name: `sb` (entry: `src/main.rs`)
- Version: 0.9.0

## Key Dependencies
| Crate | Version | Role |
|---|---|---|
| `clap` | 4.6.0 | CLI argument parsing (derive macros) |
| `qbe` | 3.0.0 | QBE SSA IR generation (primary systems backend) |
| `rust-embed` | 8.11.0 | Embed `builtin/` and `lib/` at compile time |
| `lazy_static` | 1.5.0 | Lazy initialization for embedded assets |
| `inkwell` | 0.7.1 | LLVM 10 bindings (optional, `llvm` feature) |
| `llvm-sys` | 100.2.4 | Low-level LLVM FFI (only with `llvm` feature) |
| `libc` | * | C bindings required by llvm-sys |

## Features
- Default: QBE, C, JS, x86 backends active
- `llvm` (optional): activates `inkwell` and LLVM generator

## Compilation Targets (Output Formats)
| Extension | Backend |
|---|---|
| `.js` | JavaScript (Node.js) |
| `.c` | C |
| `.ssa` | QBE SSA intermediate representation |
| `.s` | x86-64 assembly (partial) |
| LLVM IR | via `--target llvm` (no extension convention) |

## Toolchain Requirements
- Rust 1.93+ (rust-toolchain.toml)
- LLVM 10 (for `llvm` feature; installed in CI via KyleMayes/install-llvm-action)
- QBE 1.2+ binary (downloaded and compiled from source in CI: `c9x.me/compile/release/qbe-1.2.tar.xz`)
- Python 3.8+ (mdBook documentation generation)
- Node.js (optional, for `sb run`)

## Platform
- Primary: Linux
- Docker: alpine:3.13 base, statically linked via `x86_64-unknown-linux-musl`
- macOS supported for development

## Documentation
- mdBook (book.toml), source in `docs/`
- Published to antimony-lang.github.io/antimony

## Gaps & Unknowns
- No `rust-toolchain.toml` version pin for LLVM — relies on CI action
- x86 backend completeness is unknown (partial per CLAUDE.md)
- No published crate to crates.io observed (may publish as binary only)
