# Changelog

## Unreleased

**Features**

- Instead of a temporary directory, heap memory is used for compilation
- Support for binary, hexadecimal and octal number notations

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
