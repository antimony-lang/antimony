# CONVENTIONS
_Last updated: 2026-03-23_

## Summary
The codebase follows standard Rust conventions enforced by rustfmt and clippy. Naming is strict snake_case for functions/variables/files, PascalCase for types/enums/traits. All source files carry an Apache 2.0 license header. Error handling uses `Result<T, String>` throughout (no custom error types).

## Naming
| Kind | Convention | Examples |
|---|---|---|
| Functions | `snake_case` | `generate_function`, `parse_module`, `build_module` |
| Variables | `snake_case` | `in_file`, `out_file`, `token_kind`, `var_types` |
| Files | `snake_case` | `string_util.rs`, `qbe.rs`, `hast.rs` |
| Structs | `PascalCase` | `Token`, `Builder`, `QbeGenerator`, `HModule` |
| Enums | `PascalCase` | `TokenKind`, `Target`, `BinOp`, `Statement`, `Expression` |
| Traits | `PascalCase` | `Generator` |
| Constants | `UPPERCASE` | (none prominent, but convention is UPPERCASE) |
| Type aliases | `snake_case` or `PascalCase` | `SymbolTable`, `GeneratorResult`, `RcTypeDef`, `StructMeta` |
| Test functions | `test_` prefix | `test_basic_tokenizing`, `test_empty_main` |

## Code Style
- 4-space indentation (default rustfmt)
- No `.rustfmt.toml` — pure defaults
- Clippy enforced with `-D warnings` in CI; suppressed locally with `#[allow(clippy::...)]` where needed (e.g. `#[allow(clippy::needless_collect)]`)
- Trailing commas in multi-line expressions (rustfmt default)

## License Headers
Every source file begins with an Apache 2.0 copyright header:
```rust
/**
 * Copyright 20XX <Author>
 *
 * Licensed under the Apache License, Version 2.0 ...
 */
```

## Error Handling
- All fallible functions return `Result<T, String>`
- Type alias: `pub type GeneratorResult<T> = Result<T, String>;` in generator module
- Errors propagated with `?` operator; descriptive messages formed with `format!()`
- Fatal errors printed to stderr via `eprintln!("Error: {}", err)` in `main.rs`
- Position-aware errors use `highlight_position_in_file()` from `src/util/string_util.rs`
- No dedicated error type/enum — plain strings used throughout

## Import Organization
- Internal imports use `crate::` paths
- `mod.rs` files aggregate and re-export with `pub use`
- Example: `pub use last::{BinOp, Expression, Function, Module, Statement, ...}`
- Visibility: `pub` for public API, `pub(super)` for sibling access, `pub(crate)` for crate-wide, private by default

## Documentation
- `///` doc comments on public APIs and complex functions
- Inline `//` comments for non-obvious logic
- `// TODO:` and `// FIXME:` used for known issues (see CONCERNS.md)

## AST Construction Pattern
- Test helpers use builder-style free functions: `create_function()`, `create_variable()`, `block()`, `var()`
- Test helper naming: descriptive verb + noun (`create_int_expr`, `create_function_with_args`)

## Gaps & Unknowns
- No `rustfmt.toml` means style can drift if rustfmt defaults change across versions
- Some older files may predate strict clippy enforcement
