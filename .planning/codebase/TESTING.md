# TESTING
_Last updated: 2026-03-23_

## Summary
Testing is a mix of inline unit tests (lexer, parser, generator), dedicated test modules for each generator backend, and integration tests that compile example `.sb` programs end-to-end via `cargo run build`. Coverage is strongest for the JS and C backends; QBE backend has its own test module. No test framework beyond `cargo test`.

## Test Organization

```
src/
├── lexer/tests.rs              # Lexer unit tests
├── parser/tests.rs             # Parser unit tests
├── generator/tests/
│   ├── mod.rs
│   ├── js_tests.rs             # JS generator unit tests
│   ├── c_tests.rs              # C generator unit tests
│   └── qbe_tests.rs            # QBE generator unit tests
└── tests/
    └── test_examples.rs        # Integration tests (compile examples/)
```

## Unit Tests — Lexer (`src/lexer/tests.rs`)
- Test token stream output for various input snippets
- Pattern: call `tokenize(source)`, assert `TokenKind` sequence
- Naming: `test_basic_tokenizing`, etc.

## Unit Tests — Parser (`src/parser/tests.rs`)
- Test AST construction from source strings
- Uses builder helpers: `builtins()`, `user_code()`, `block()`, `var()`, `module()`, `func()`
- Naming: `test_parse_function_with_return`, `test_empty_main`, etc.

## Unit Tests — Generators (`src/generator/tests/`)
Each generator test file constructs AST nodes directly (no parsing) and asserts on the emitted output string.

**QBE tests (`qbe_tests.rs`)** use helpers:
- `create_function(name, ret_type, body)`
- `create_function_with_args(name, arguments, ret_type, body)`
- `create_variable(name, type)`
- `create_int_expr(value)`, `create_bool_expr(value)`, `create_str_expr(value)`, `create_var_expr(name)`
- `create_binop_expr(lhs, op, rhs)`, `create_call_expr(fn_name, args)`
- `normalize_qbe(output)` — strips whitespace/empty lines for robust comparison

All test functions are inside `#[cfg(test)] mod tests { ... }` blocks.

## Integration Tests (`src/tests/test_examples.rs`)
- `test_directory(dir_in)` — iterates all `.sb` files in a directory
- Runs `cargo run build <file> -o <out>.js` as a subprocess
- If Node.js is installed, also executes the compiled output and asserts exit code 0
- Currently targets JS backend only (`out_file_suffix = ".js"`)
- Tests run against the `examples/` directory

## CI Test Execution
- `cargo test` runs in CI with both LLVM 10 and QBE 1.2 binary installed
- All 4 CI jobs (check, test, fmt, clippy) run on every push/PR to ubuntu-latest

## What Is NOT Tested
- QBE integration tests (no end-to-end `.sb → .ssa → binary` test)
- C backend integration (no compile + run)
- x86 backend (no tests observed)
- LLVM backend (no tests observed; generator is incomplete)
- Multi-file programs with imports
- Error recovery / error message quality

## Gaps & Unknowns
- No coverage metrics collected
- No property-based or fuzz testing
- Integration tests depend on `cargo run` (slow) rather than a pre-built binary
- QBE unit tests test IR string output but not execution correctness
