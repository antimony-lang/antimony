# INTEGRATIONS
_Last updated: 2026-03-23_

## Summary
Antimony integrates with QBE as the primary code generation backend (both as a Rust crate and as an external binary), LLVM 10 optionally, and standard C/JS toolchains as pass-through targets. CI runs on GitHub Actions with four jobs.

## QBE (Primary Systems Backend)
- **Rust crate**: `qbe = "3.0.0"` — used in `src/generator/qbe.rs` to construct SSA IR programmatically
- **External binary**: The `qbe` binary must be installed separately (CI installs from source: `qbe-1.2.tar.xz`)
- The Rust crate emits QBE IL text; the external binary then assembles it to native code
- `QbeGenerator` uses `qbe::Module`, `qbe::Function`, `qbe::TypeDef` from the crate

## LLVM (Optional)
- Feature flag: `llvm` in Cargo.toml
- Uses `inkwell 0.7.1` (safe LLVM wrappers) + `llvm-sys 100.2.4` (raw FFI)
- Requires LLVM 10 headers/library at build time
- `src/generator/llvm.rs` — largely incomplete (multiple `todo!()` calls)
- Not part of default build

## C Transpilation
- `src/generator/c.rs` — generates C source code
- No external C compiler invoked by the compiler itself; user runs cc/gcc manually
- No C standard library headers emitted; relies on user toolchain

## JavaScript
- `src/generator/js.rs` — generates JavaScript (Node.js compatible)
- `sb run` uses `cargo run build ... -o out.js && node out.js`
- Standard library (`lib/`) embedded via `rust-embed` and appended automatically for JS target

## Embedded Assets
- `rust-embed 8.11.0` embeds `builtin/` and `lib/` directories at compile time
- Accessed via `crate::Lib` in builder
- Standard library appended for JS and QBE targets only (not C or x86)

## GitHub Actions CI
Four parallel jobs on every push/PR:
| Job | What it does |
|---|---|
| `check` | `cargo check` with LLVM 10 |
| `test` | `cargo test` with LLVM 10 + QBE 1.2 binary installed |
| `fmt` | `rustfmt --check` |
| `clippy` | `clippy --all-targets -D warnings` |

**Note**: CI uses `actions-rs/cargo@v1` (deprecated action) and `actions/checkout@v2` (older version). Both still work but are behind current best practices.

## Docker
- Base: `alpine:3.13`
- Static linking target: `x86_64-unknown-linux-musl`
- No external runtime dependencies beyond the `sb` binary

## Gaps & Unknowns
- No release automation / publishing pipeline observed
- No integration with package managers (Homebrew, apt, etc.)
- QBE binary version mismatch risk: crate uses 3.0.0 API but CI installs QBE 1.2 binary
