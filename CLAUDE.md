<!-- GSD:project-start source:PROJECT.md -->
## Project

**Antimony**

Antimony is a personal-project compiled programming language with a multi-backend architecture (JS, C, QBE, LLVM, x86). The immediate goal is to mature the QBE backend until two milestones are reached: a bootstrapped compiler (the Antimony compiler written in Antimony and compiled via QBE) and a Doom port written in Antimony.

**Core Value:** The QBE backend must become capable enough that real systems programs — including the compiler itself — can be written in Antimony and compiled correctly.

### Constraints

- **Tech Stack**: QBE as the primary backend — all systems-level work must target QBE
- **Bootstrap**: The bootstrapped compiler must be a full rewrite (not a subset), compiled via QBE
- **Personal project**: No team, no deadlines — prioritize learning and correctness over velocity
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Languages
- Rust 1.93 - Core compiler implementation, lexer, parser, code generators, CLI
- Python 3.8 - Documentation build tooling
- C - Backend/intermediate language target for compilation
- JavaScript - Backend/intermediate language target for compilation
- SSA (QBE IL) - Intermediate representation for code generation
- x86 Assembly - Backend/intermediate language target (partial)
## Runtime
- Linux (alpine:3.13 for Docker)
- Targets: JavaScript (Node.js), C, QBE, x86 assembly
- Cargo (Rust package manager)
- Lockfile: `Cargo.lock` (present)
## Frameworks
- clap 4.6.0 - CLI argument parsing with derive macros
- rust-embed 8.11.0 - Embed static files (library and builtin code) at compile time
- Cargo - Build system and project management
- mdBook - Documentation generation (with Markdown source)
- mdbook 2.0+ - Book generator for documentation
- qbe 3.0.0 - QBE (QBE Backend Equator) for intermediate code generation
- inkwell 0.7.1 (optional feature: llvm10-0) - LLVM bindings (optional, behind `llvm` feature flag)
- rustfmt - Code formatting (enforced in CI)
- clippy - Linting tool (enforced in CI with `-D warnings`)
## Key Dependencies
- qbe 3.0.0 - QBE backend for SSA/IR generation, required for compilation
- clap 4.6.0 - CLI interface, essential for compiler invocation
- rust-embed 8.11.0 - Embeds builtin libraries and standard library at compile time, required for runtime
- lazy_static 1.5.0 - Lazy static initialization
- llvm-sys 100.2.4 - Low-level LLVM FFI (used only when `llvm` feature enabled)
- libc - C library bindings (required by llvm-sys)
## Configuration
- No environment variables required for basic operation
- Dockerfile uses static linking via musl target for portability
- `Cargo.toml` - Rust manifest with dependencies and metadata
- `rust-toolchain.toml` - Specifies Rust 1.93 with minimal profile
- `.github/workflows/ci.yml` - GitHub Actions CI pipeline
- `book.toml` - mdBook documentation configuration
## Platform Requirements
- Rust 1.93+ (managed via `rust-toolchain.toml`)
- LLVM 10 (required for LLVM backend compilation tests, installed in CI)
- QBE 1.2+ (downloaded and compiled from source in CI)
- Python 3.8+ (for documentation building)
- Standard POSIX tools (git, shell, etc.)
- No external runtime dependencies beyond standard C library
- Docker deployment uses alpine:3.13 base
- Compiled binary can be statically linked with x86_64-unknown-linux-musl target
## Compile Targets
- `.c` - C language output (backend)
- `.js` - JavaScript output (backend)
- `.ssa` - QBE SSA intermediate representation (backend)
- `.s` - x86-64 assembly output (backend, partial)
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

## Naming Patterns
- Snake case for module files: `string_util.rs`, `parser.rs`, `lexer.rs`
- Test files placed inline with `mod tests;` declarations or in separate `tests/` directory
- Generator implementation files named by target: `js.rs`, `c.rs`, `qbe.rs`, `x86.rs`, `llvm.rs`
- Snake case for all functions: `test_basic_tokenizing()`, `parse_module()`, `generate_function()`, `match_token()`
- Test functions prefixed with `test_`: `test_empty_main()`, `test_parse_function_with_return()`, `test_generate_block_empty()`
- Helper functions in tests prefixed with descriptive names: `builtins()`, `user_code()`, `block()`, `var()`, `module()`, `func()`
- Private helper functions use leading underscore if needed for clarity
- Snake case for all variables: `in_file`, `out_file`, `token_kind`, `var_types`, `dir_out`
- Type-aliased collections are descriptive: `SymbolTable = HashMap<String, Option<Type>>`
- PascalCase for structs: `Token`, `Parser`, `Module`, `Function`, `Variable`, `Statement`
- PascalCase for enums: `TokenKind`, `Keyword`, `Expression`, `BinOp`, `Target`
- PascalCase for trait names: `Generator`
- UPPERCASE for constants
- Type aliases in snake_case when they're type definitions: `pub type GeneratorResult<T> = Result<T, String>;`
## Code Style
- Rust standard formatting (inferred from codebase)
- 4-space indentation observed throughout
- No `.rustfmt.toml` file in repository; uses default Rust formatting
- Clippy is integrated - code uses `#[allow(clippy::...)]` annotations where needed
- Example: `#[allow(clippy::needless_collect)]` in `src/parser/parser.rs:33`
## Import Organization
- Uses relative `crate::` paths for internal imports
- Modules declared with `mod` and re-exported with `pub use`
- Example in `src/ast/mod.rs`: `pub use last::{BinOp, Expression, Function, Module, Statement, StructDef, SymbolTable, Variable};`
## Error Handling
- Results return `Result<T, String>` for error messages
- Error messages are descriptive strings propagated up the call stack
- Type alias pattern: `pub type GeneratorResult<T> = Result<T, String>;` used consistently in generator modules
- Errors are formatted with context using `map_err()` closures
- Functions use early return with `?` operator
- Main entry point catches errors and exits with code 1
- Semantic errors in parser include file context via `highlight_position_in_file()`
## Logging
- Only used for fatal errors to stderr
- Main error handler in `src/main.rs:69`: `eprintln!("Error: {}", err);`
- No logging framework for debug/info level logging
## Comments
- License header on all source files (Apache 2.0)
- Function-level documentation with `///` for public APIs and complex functions
- Inline comments for non-obvious logic or known limitations
- Rust uses `///` for documentation comments on public items
- Example from `src/parser/parser.rs`:
- Documentation comments generate rustdoc automatically
- TODO comments used to mark known issues: `// TODO: do something better`
## Function Design
- Functions accept owned types or references based on lifetime needs
- Builder pattern used for complex AST construction (see test helpers)
- Helper functions in tests build up complex AST nodes progressively
- Use `Result<T, String>` for fallible operations
- Use `Option<T>` for nullable returns
- Unit type `()` for operations that succeed or fail without returning data
## Module Design
- Modules declare submodules with `pub mod`
- Re-export important types with `pub use`
- Internal implementation hidden with private items
- `mod.rs` files aggregate module exports
- Example in `src/ast/mod.rs`:
- `pub` for public APIs
- `pub(super)` for sibling module access in parser
- `pub(crate)` for internal crate-wide access
- Private by default (no `pub` keyword)
## Code Organization
- Implemented on types with `impl TraitName for Type`
- Each type may have multiple trait implementations
- Snake case field names in all structs
- Public fields for data structures like `Token`, `Function`, `Variable`
- Fields grouped logically (positions, names, types together)
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## Pattern Overview
- Source code → Tokens → High-level AST → Low-level AST → Multiple code generation targets
- Clear separation between parsing/analysis and code generation phases
- Pluggable backend architecture supporting multiple target languages (JavaScript, C, QBE, x86, LLVM)
- Two-level AST representation enabling complex-to-simple lowering transforms
- Module system with import resolution and circular import detection
## Layers
- Purpose: Converts raw source code strings into a sequence of tokens
- Location: `src/lexer/`
- Contains: Token definitions, `TokenKind` enum (keywords, operators, literals), `Cursor` for character-by-character scanning
- Depends on: Standard library only
- Used by: Parser
- Purpose: Transforms token stream into a High-level Abstract Syntax Tree (HModule)
- Location: `src/parser/`
- Contains: Parser state machine, parsing rules, type inference pass, error recovery
- Depends on: Lexer output (tokens), AST types
- Used by: Builder, receives HModule for downstream processing
- Purpose: Annotates AST nodes with type information for untyped expressions
- Location: `src/parser/infer.rs`
- Contains: Symbol table construction, type propagation logic
- Depends on: HModule from parser
- Used by: AST transformation and code generators
- Purpose: Provides intermediate representations at different abstraction levels
- Location: `src/ast/`
- High-level AST (`hast.rs`): HModule, HFunction, HStatement, HExpression - contains high-level constructs (match statements, for-in loops)
- Low-level AST (`last.rs`): Module, Function, Statement, Expression - simplified constructs ready for code generation (no match, for becomes while)
- Contains: Type definitions, symbol tables, transformation logic
- Depends on: Lexer types
- Used by: Transformer, all code generators
- Purpose: Lowers high-level AST to low-level AST by desugaring complex constructs
- Location: `src/ast/transform.rs`
- Contains: AstTransformer with methods to lower HStatement → Statement, HExpression → Expression
- Depends on: HAST and LAST type definitions
- Used by: Builder before code generation
- Purpose: Orchestrates the compilation pipeline - handles file loading, import resolution, module merging
- Location: `src/builder/mod.rs`
- Contains: Builder struct, build() method, module recursion with circular import prevention, standard library inclusion
- Depends on: Lexer, Parser, AST types, file I/O
- Used by: Command handlers (build, run)
- Purpose: Transforms low-level AST to target language code
- Location: `src/generator/`
- Contains: Multiple generator implementations (js.rs, c.rs, qbe.rs, x86.rs, llvm.rs)
- Pattern: Each generator implements `Generator` trait with single `generate(Module) -> String` method
- Depends on: LAST module representation, builtin code
- Used by: Builder to produce final output
- Purpose: Entry points for user-facing CLI commands
- Location: `src/command/`
- Contains: build.rs (handles file I/O, orchestrates Builder), run.rs (executes compiled output)
- Depends on: Builder, code generators
- Used by: main.rs after CLI argument parsing
## Data Flow
- HModule accumulates parsed functions, structs, globals, and imports during recursive module loading
- Parser maintains state: current token, previous token, peeked token buffer, raw source for error reporting
- Type information flows through symbol tables (SymbolTable = HashMap<String, Option<Type>>)
- Transformation is stateless - pure conversion of HAST to LAST structures
## Key Abstractions
- Purpose: Represents individual lexical units (operators, keywords, literals, identifiers)
- Examples: `TokenKind::Keyword(Keyword::If)`, `TokenKind::Literal(Value::Int)`, `TokenKind::Identifier(String)`
- Pattern: Enum-based classification enabling pattern matching in parser
- Purpose: Container for parsed/transformed program with all functions and types
- Examples: `src/ast/hast.rs` (high-level), `src/ast/last.rs` (low-level)
- Pattern: Separates concerns - high-level for language features, low-level for code generation
- Purpose: Pluggable interface for code generation backends
- Examples: `JsGenerator`, `CGenerator`, `QbeGenerator`, `X86Generator`, `LlvmGenerator`
- Pattern: Implements `Generator` trait with single `generate()` method taking Module
- Purpose: Recursive tree representation of program logic
- Examples: `Statement::If { condition, body, else_branch }`, `Expression::BinOp { lhs, op, rhs }`
- Pattern: Enum variants enable exhaustive pattern matching for transformations and code generation
- Purpose: Stateful accumulation of modules during import resolution
- Examples: `Builder::build_module()` recursively loads and parses imported files
- Pattern: Maintains modules vector, tracks seen imports to prevent circular inclusion
## Entry Points
- Location: `src/main.rs` - main() function
- Triggers: User runs `sb` command with subcommand
- Responsibilities: Parse CLI arguments via clap, dispatch to command handlers (build or run)
- Location: `src/command/build.rs` - build() function
- Triggers: `sb build` subcommand
- Responsibilities: Create Builder, orchestrate compilation, write output file
- Location: `src/command/run.rs` - run() function
- Triggers: `sb run` subcommand
- Responsibilities: Build to JS target, execute with Node.js
## Error Handling
- All major operations return `Result<T, String>` enabling `?` operator for error propagation
- Parser recovers from tokenization errors, reports position information via `src/util/string_util::highlight_position_in_file()`
- Builder validates file existence and module imports before parsing
- Generators can fail on type inconsistencies (e.g., JsGenerator expects transformed AST structure)
- Circular import detection prevents infinite loops via `seen` vector tracking in build_module()
## Cross-Cutting Concerns
- Lexer validates character sequences match token kinds
- Parser enforces syntax rules and tracks token position for error reporting
- Type inference validates function signatures and variable types
- No dedicated validation layer - constraints encoded in AST structure and parser rules
- Imports are relative paths from importing file location
- Circular imports prevented by tracking seen modules
- Standard library (`lib/` directory) automatically appended for JS and QBE targets
- Builtin functions embedded via rust-embed from `builtin/` directory
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
