# Changelog

## Unreleased

**Fixes**

- Fixed array access in member variable expressions ([#123](https://github.com/antimony-lang/antimony/pull/123))

**Maintenance**

- Add multi-level AST structure with High-level AST (HAST) and Low-level AST (LAST) ([#124](https://github.com/antimony-lang/antimony/pull/124))
- Bump dependency [qbe](https://crates.io/crates/qbe) from `2.5.1` to `3.0.0` ([#131](https://github.com/antimony-lang/antimony/pull/131))

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
