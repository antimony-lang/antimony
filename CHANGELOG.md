# Changelog

## Unreleased

**Features**

- QBE: Fix type inference for comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`) and boolean (`&&`, `||`) operators ([#167](https://github.com/antimony-lang/antimony/issues/167))
- QBE: Implement `len()` as a built-in intrinsic that reads the array-length header ([#171](https://github.com/antimony-lang/antimony/pull/171))
- QBE: Support field access on function call results, e.g. `user_stub().first_name` ([#161](https://github.com/antimony-lang/antimony/pull/161))
- QBE: Auto-coerce `int` arguments to `string` parameters via `_int_to_str` ([#161](https://github.com/antimony-lang/antimony/pull/161))
- QBE: Implement for-in loops over arrays ([#152](https://github.com/antimony-lang/antimony/pull/152))
- QBE: Implement struct methods and `self` — methods are generated as `StructName__methodName(self: Long, ...)` and method calls on struct values are lowered correctly ([#151](https://github.com/antimony-lang/antimony/pull/151))
- QBE: Implement array element reads and writes (`arr[i]`, `arr[i] = v`) ([#149](https://github.com/antimony-lang/antimony/pull/149))
- QBE: Add builtins (`_printf`, `_exit`), stdlib support, and string concatenation for native executables ([#147](https://github.com/antimony-lang/antimony/pull/147))
- QBE: Support `Type::Any` with argument widening for native executables ([#150](https://github.com/antimony-lang/antimony/pull/150))
- stdlib: Add `math` module with `min`, `max`, `abs`, `pow`, and `clamp` functions ([#160](https://github.com/antimony-lang/antimony/pull/160))
- stdlib: Add `string` module with `str_len`, `repeat`, and `to_int` functions ([#160](https://github.com/antimony-lang/antimony/pull/160))
- stdlib: Extend `array` module with `sum`, `contains`, `min_array`, `max_array`, `first`, and `last` functions ([#160](https://github.com/antimony-lang/antimony/pull/160))
- stdlib: Add `read_line()` to `io` module for reading a line from stdin ([#160](https://github.com/antimony-lang/antimony/pull/160))
- Add 15 unit tests for the type inference module (`infer.rs`) covering literals, variables, arrays, function calls, builtins, binary operators, structs, for-loops, array access, nested if/else, match arms, and explicit type preservation ([#186](https://github.com/antimony-lang/antimony/issues/186))

**Fixes**

- Lexer: fix infinite loop in `comment()` when EOF is reached without a trailing newline ([#199](https://github.com/antimony-lang/antimony/pull/199))
- Lexer: fix infinite loop in `eat_string()` on unterminated string literals at EOF ([#199](https://github.com/antimony-lang/antimony/pull/199))
- QBE: Use native `blit` instruction instead of `memcpy` call for aggregate struct field copies ([#157](https://github.com/antimony-lang/antimony/issues/157))
- QBE: Void function calls no longer assign to a temporary — `call $fn()` is now emitted instead of the invalid `%tmp =w call $fn()` ([#156](https://github.com/antimony-lang/antimony/issues/156))
- QBE: Fix `Bool` type mapped to `Byte` (`b`) in `get_type()` — booleans now map to `Word` (`w`), consistent with how bool literals and comparisons are already emitted ([#155](https://github.com/antimony-lang/antimony/issues/155))
- Type inference: seed variable map with function parameter types so expressions referencing parameters can be inferred ([#163](https://github.com/antimony-lang/antimony/issues/163))
- QBE: Fix array indexing for uninitialized sized arrays — `let foo: int[5]` now allocates memory correctly ([#174](https://github.com/antimony-lang/antimony/pull/174))
- QBE: Infer for-in loop variable type from array element type — `for x in arr {}` no longer requires an explicit type annotation on `x` ([#168](https://github.com/antimony-lang/antimony/issues/168))
- QBE: Fix expression-bodied functions with string concat (e.g. `fn greet(name: string) = "Hello " + name`) producing invalid SSA and being emitted as void functions ([#172](https://github.com/antimony-lang/antimony/pull/172))
- QBE: Fix false "does not return in all code paths" error for complete `if/else if/else` chains where all branches return ([#170](https://github.com/antimony-lang/antimony/pull/170))
- QBE: Fix type inference for `len()` and other cross-module function calls — inference now runs after module merge so the full symbol table is visible ([#171](https://github.com/antimony-lang/antimony/pull/171))
- Type inference: recurse into nested blocks (while/if/for/match) and resolve array-access element types ([#171](https://github.com/antimony-lang/antimony/pull/171))
- Parser: fix operator precedence — `a % b == 0` now correctly parses as `(a % b) == 0` ([#161](https://github.com/antimony-lang/antimony/pull/161))
- Parser: treat `_` in match arms as a catch-all else branch instead of a variable lookup ([#161](https://github.com/antimony-lang/antimony/pull/161))
- QBE: fix `_exit` infinite recursion by using `_Exit()` instead of `exit()` ([#161](https://github.com/antimony-lang/antimony/pull/161))
- QBE: Fix arithmetic type propagation — use the wider operand type instead of hardcoded `Word`, and widen `Byte` (bool) operands with `extub` when needed ([#159](https://github.com/antimony-lang/antimony/pull/159))
- QBE: Clean up `resolve_field_access` — use name-based struct lookup instead of fragile QBE type equality scan ([#159](https://github.com/antimony-lang/antimony/pull/159))
- QBE: Clean up temp files after `run_qbe` execution ([#159](https://github.com/antimony-lang/antimony/pull/159))
- QBE: Fix function return types and struct aggregate types ([#148](https://github.com/antimony-lang/antimony/pull/148))
- Fixed array access in member variable expressions ([#123](https://github.com/antimony-lang/antimony/pull/123))
- stdlib: Fix `rev()` bug in `array` module (incorrect loop logic) ([#160](https://github.com/antimony-lang/antimony/pull/160))

**Maintenance**

- Add property-based tests for the lexer using `proptest` — no-panic on arbitrary input, round-trip reconstruction, and integer literal tokenization ([#199](https://github.com/antimony-lang/antimony/pull/199))
- QBE: Run all examples in QBE integration tests instead of a hardcoded subset ([#163](https://github.com/antimony-lang/antimony/issues/163))
- QBE: Add integration tests and CI support — compile and run examples and test cases through the full QBE pipeline (compile → qbe → gcc → execute) ([#145](https://github.com/antimony-lang/antimony/issues/145))
- Update deprecated GitHub Actions (`peaceiris/actions-mdbook@v1` → `v2`, `actions/setup-python@v2` → `v5`) to fix docs deployment ([#153](https://github.com/antimony-lang/antimony/pull/153))
- Remove unsupported `multilingual` field from `book.toml` to fix mdBook 0.5.2 compatibility ([#158](https://github.com/antimony-lang/antimony/pull/158))
- Add multi-level AST structure with High-level AST (HAST) and Low-level AST (LAST) ([#124](https://github.com/antimony-lang/antimony/pull/124))
- Bump dependency [qbe](https://crates.io/crates/qbe) from `2.5.1` to `3.0.0` ([#131](https://github.com/antimony-lang/antimony/pull/131))
- Replace unmaintained `structopt` with `clap` ([#132](https://github.com/antimony-lang/antimony/pull/132))
- Pin Rust toolchain to `1.93` in `rust-toolchain.toml` ([#133](https://github.com/antimony-lang/antimony/pull/133))
- Remove x86 and LLVM backends -- only C, JS, and QBE compilation targets remain ([#195](https://github.com/antimony-lang/antimony/pull/195))

## v0.9.0 (2025-04-18)

**Features**

- QBE: Added proper memory alignment for struct fields based on field types ([#61](https://github.com/antimony-lang/antimony/pull/61))
- Improved error reporting with more descriptive messages and helpful hints for common parsing errors ([#118](https://github.com/antimony-lang/antimony/pull/118))
- Added proper Display trait implementation for lexer tokens to improve error message formatting ([#118](https://github.com/antimony-lang/antimony/pull/118))

**Fixes**

- Fixed parsing of binary operations in inline function expressions ([#109](https://github.com/antimony-lang/antimony/pull/109))
- Fixed parsing of binary operations after function calls ([#111](https://github.com/antimony-lang/antimony/pull/111))
- QBE: Fixed struct field access and memory layout for nested structs ([#61](https://github.com/antimony-lang/antimony/pull/61))

**Maintenance**

- Refactored command execution for better error handling and code organization
- Remove regex dependency
- Bump dependency [qbe](https://crates.io/crates/qbe) from `1.0.0` to `2.4.0`
- Bump dependency [rust-embed](https://crates.io/crates/rust-embed) from `5.7.0` to `8.7.0`

## v0.8.0 (2024-04-05)

**Features**

- Support for shorthand function bodies ([#94](https://github.com/antimony-lang/antimony/pull/94))

**Maintenance**

- Bump dependency [structopt](https://crates.io/crates/structopt) from `0.3.21` to `0.3.26`
- Bump dependency [inkwell](https://crates.io/crates/inkwell) from `0.1.0-beta.2` to `0.4.0`
- Bump dependency [regex](https://crates.io/crates/regex) from `1.5.5` to `1.10.4`

## v0.7.0 (2022-06-15)

**Changes**

- Arrays now have a fixed capacity

**Features**

- Instead of a temporary directory, heap memory is used for compilation
- Support for binary, hexadecimal and octal number notations
- Support for `_` character in integers (E.g. `1_000_000`)
- Parser errors have been improved in consistency and readability
- Compile to stdout by using the `-o -` flag
- Proper support for utf-8
- Initial support for QBE backend

**Fixes**

- Allow constructor expressions as function arguments
- Fix `self` keyword inside statement

## v0.6.0 (2021-02-28)

**Changes**

- Comma separation for struct fields has been removed

**Features**

- Struct methods (#19)
- Compile-backend will be determined based on file extension (#20, #21)

**Fixes**

- Fixed a bug where strings were terminated using both `"` and `'`
- Fixed circular imports for modules
- Fixed structs not being imported from other modules

## v0.5.1 (2021-02-25)

Sabre is now Antimony!

**Changes**

- "sabre" was replaced with "sb" (E.g. run `sb run main.sb` to run a program)

## v0.5.0 (2021-02-23)

**Features**

- Match statements (#15)
- Modules and imports (#17)
- Support for Docker
- Support for Arch Linux build system (Thanks Alex)

**Fixes**

- Fixed a bug with nested expressions and arithmetic operations

**Documentation**

- Added some in-repo technical documentation
- Minor fixes

## v0.4.0 (2021-02-20)

This release introduces the concept of structs, alongside many improvements to the documentation.

**Features**

- Assignment operators (#10)
- Structs (#12)

**Fixes**

None

**Documentation**

- Fixed some typose and broken links
- Document boolean values
- Added this changelog!

## v0.3.0 (2021-02-12)

This release adds type inference to Antimony. There are also a lot of improvements in terms of documentation. The docs are now at a state that can be considered "usable".

**Features**

- Type inference
- The `any` type
- First attempt of LLVM backend

**Fixes**

- Fixed an error when printing numbers

**Documentation**

- Added documentation for for loops
- Added documentation for while loops
- Documented LLVM backend
- Documented comments
- Updated contributing guidelines

## v0.2.1 (2021-02-06)

**Fixes**

- Fixed an issue where nested expressions where not compiled correctly

## v0.2.0 (2021-02-06)

This version introduces a lot of improvements regarding loops and arrays.

**Features**

- Support for nested arrays
- `break` and `continue` statements

**Documentation**

- Link to our matrix channel in README
- Install Antimony via Cargo

## v0.1.1 (2021-02-06)

Follow-up release that fixes some issues with the published crate.

## v0.1.0 (2021-02-06)

This release is the first to be published to crates.io. The crate is called [antimony-lang](https://crates.io/crates/antimony-lang).

**Features**

- Uninitialized variables
- For loops

**Fixes**

None

**Documentation**

- Functions fully documented

## v0.0.4 (2020-12-18)

This release tries to lay the groundwork of a possible C backend.

**Features**

- An unstable, opt-in C backend
- `len()` standard library function
- `rev()` standard library function
- Function return types

**Fixes**

- Booleans as function parameters

**Documentation**

- A lot of improvements

## v0.0.3 (2020-12-10)

This release adds new vital features to the language.

**Features**

- While loops
- Boolean type
- Variable assignments
- Basic standard library

## v0.0.2 (2020-12-09)

Direct follow-up release that didn't add or remove anything

## v0.0.1 (2020-12-09)

Initial release with basic featureset.

**Full shortlog**

```
Garrit Franke (74):
      Initial commit
      Add license
      Add curly braces
      Parse functions
      Fix function parsing
      Update math example
      Refactor TokenType
      Implement return statement
      Fix keyword recognition
      Fix test compilation
      Fix tests and comments
      Add variable declarations
      Implement returning variables
      Pretty print AST output
      Add strings
      Rename flex -> antimony
      Fix example filename
      Add parser tests
      Add multiple functions test
      Add token positions
      Add token positions
      Allow empty returns
      Add x86 generator scaffolding
      Generate assembly
      Add JS generator
      Fix warnings
      Implement return generation
      Refactor x86 generator
      Print result of main for js target
      Fix infinite loop when parsing strings
      Add CI
      Add function arguments
      Add function arguments
      Tokenize Comma
      Fix return with expression
      Remove uneeded compount statement
      Add math operations
      Fix parsing of complex compound expressions
      Clean up expression parsing
      Refactor function call parsing
      Change function declaration syntax
      Add greeter example
      Add fib example
      Add basic conditionals; remove semicolons
      Allow multiple statements in if conditional
      Add TODO file
      Add js generator for variable declarations
      Add remaining comparison operators
      Add Readme
      Add backend-state to README
      Add CLI TODO
      Add error reporting TODO
      Fix typo
      Add builds.sr.ht badge
      Add basic CLI
      Revert "Change function declaration syntax"
      Fix production build
      Fix examples
      Fix readme
      Fix compound op with identifier first
      Fix fib example
      Add conditional else if branch
      Generalize block into own statement
      Add else branch
      Add copyright notices
      Fix warnings
      Add integer arrays
      Clean up error handling
      Refactor parser module structure
      Fix warnings
      Add docs
      Add placeholder for documentation
      docs: add placeholder for CLI
      docs: add placeholders for developers
```
