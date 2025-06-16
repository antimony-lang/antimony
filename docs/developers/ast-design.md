# AST Design

The Antimony compiler uses a sophisticated two-level Abstract Syntax Tree (AST) design to balance language expressiveness with code generation simplicity.

## Overview

The compiler transforms source code through two distinct AST representations:

1. **HAST (High-level AST)** - Preserves source-level semantics and complex language constructs
2. **LAST (Low-level AST)** - Provides simplified constructs optimized for code generation

This separation allows the compiler to support rich language features while keeping backend implementations straightforward.

## High-level AST (HAST)

The High-level AST (`src/ast/hast.rs`) represents code as directly parsed from source files. It maintains the original structure and semantic richness of the source language.

### Key Characteristics

- **Source fidelity** - Preserves the original intent and structure of the code
- **Rich constructs** - Includes complex language features like pattern matching
- **Analysis-friendly** - Designed for type checking, semantic analysis, and high-level optimizations

### Notable Features

- **Match statements** with pattern matching and multiple arms
- **Complex expressions** that may require multi-step lowering
- **High-level control flow** constructs
- **Structured data** with full semantic information

### Usage

HAST is used during the early phases of compilation:
- **Parsing** - Direct output from the parser
- **Semantic analysis** - Type checking and validation
- **High-level optimizations** - Source-level transformations
- **Error reporting** - Maintains source location information

## Low-level AST (LAST)

The Low-level AST (`src/ast/last.rs`) represents code ready for backend code generation. It contains only simple constructs that map directly to target languages.

### Key Characteristics

- **Backend-friendly** - Every construct has a direct mapping to target languages
- **Simplified structure** - Complex features are broken down into basic operations
- **Code generation ready** - Optimized for easy translation to C, JavaScript, etc.

### Notable Features

- **If-else chains** instead of match statements
- **Simple expressions** with explicit operation sequences
- **Flat control flow** structures
- **Basic data operations** without high-level abstractions

### Usage

LAST is used during the final phases of compilation:
- **Code generation** - Direct input to backend generators
- **Low-level optimizations** - Target-specific improvements
- **Final transformations** - Backend-specific adjustments

## Transformation Process

The transformation from HAST to LAST is handled by the `AstTransformer` in `src/ast/transform.rs`. This process performs several key lowering operations:

### Match Statement Lowering

The most significant transformation is converting match statements into if-else chains:

```rust
// HAST: High-level match
match value {
    1 => action_one(),
    2 => action_two(),
    _ => default_action(),
}

// LAST: Lowered if-else chain
if value == 1 {
    action_one()
} else if value == 2 {
    action_two()
} else {
    default_action()
}
```

### Expression Simplification

Complex expressions are broken down into simpler operations that backends can easily handle.

### Structure Flattening

Nested and complex structures are flattened to ensure consistent code generation across all backends.

## Benefits of Two-Level Design

### Language Expressiveness

The high-level AST allows Antimony to support rich language features without constraining the implementation:

- **Pattern matching** - Full-featured match expressions
- **Complex control flow** - Sophisticated branching and looping constructs
- **Rich data types** - Advanced type system features

### Backend Simplicity

The low-level AST ensures that code generators remain simple and maintainable:

- **Consistent interface** - All backends work with the same simplified constructs
- **Easier implementation** - New backends don't need to handle complex language features
- **Better optimization** - Target-specific optimizations can focus on simple patterns

### Maintainability

The separation of concerns improves compiler maintainability:

- **Clear boundaries** - High-level and low-level concerns are separated
- **Independent evolution** - Language features and code generation can evolve independently
- **Easier debugging** - Issues can be isolated to specific compilation phases

## Implementation Details

### Module Structure

```
src/ast/
├── mod.rs          # Public API and re-exports
├── hast.rs         # High-level AST definitions
├── last.rs         # Low-level AST definitions
├── transform.rs    # HAST → LAST transformation
└── types.rs        # Shared type definitions
```

### Type Relationships

Both AST levels share common type definitions where appropriate, but use different structural representations for statements and expressions that require transformation.

### Error Handling

The transformation process includes comprehensive error handling to catch issues during the lowering phase, ensuring that invalid high-level constructs are detected before code generation.

## Future Considerations

The two-level design provides flexibility for future enhancements:

- **New language features** can be added to HAST without affecting backends
- **Additional optimization passes** can be inserted between the AST levels
- **Multiple lowering strategies** can be implemented for different optimization profiles
- **Incremental compilation** can cache transformations at different levels