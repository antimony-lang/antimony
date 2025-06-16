# AST

This module contains the node types of the Antimony AST.

The most important node is `Module`, which is the abstract representation of a Antimony file.

## AST Levels

Antimony uses a two-level AST design to separate high-level language constructs from low-level code generation:

### HAST (High-level AST) - `hast.rs`

The High-level AST represents code as directly parsed from source files. It contains rich language constructs that don't necessarily map directly to simple backend operations:

- **Match statements** - Pattern matching with multiple arms and else clauses
- **Complex expressions** - High-level constructs that may require transformation
- **Source-level semantics** - Preserves the original structure and intent of the code

HAST is used during:
- Parsing and semantic analysis
- Type checking and validation
- High-level optimizations

### LAST (Low-level AST) - `last.rs`

The Low-level AST represents code ready for code generation. It contains only simple constructs that map directly to backend targets:

- **If-else chains** - Match statements are lowered to nested if-else structures
- **Simple expressions** - All complex constructs are broken down to basic operations
- **Backend-friendly structure** - Designed for easy translation to C, JavaScript, etc.

LAST is used during:
- Code generation for various backends
- Low-level optimizations
- Final compilation steps

### Transformation

The `transform.rs` module handles conversion from HAST to LAST:

- **Match lowering** - Transforms match statements into if-else chains with equality checks
- **Expression simplification** - Breaks down complex expressions into simpler forms
- **Structure flattening** - Ensures all constructs can be easily handled by code generators

This two-level design allows Antimony to support rich language features while keeping the code generation backends simple and maintainable.
