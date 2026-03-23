# Feature Landscape

**Domain:** Self-hosting compiler (Antimony compiler written in Antimony, compiled via QBE)
**Researched:** 2026-03-23
**Overall Confidence:** HIGH (based on direct codebase analysis of what the Rust compiler does, mapped against Antimony's current language capabilities)

## Table Stakes

Features the language **must** have or self-hosting is impossible. These are derived by analyzing what the existing Rust compiler actually uses, component by component.

### Data Structure & Memory Features

| Feature | Why Required | Complexity | Current Status | Notes |
|---------|-------------|------------|----------------|-------|
| Dynamic arrays (growable vectors) | Token list, AST node children, scope stacks, error messages -- every compiler phase accumulates variable-length data | High | Arrays exist but only fixed/literal; no push/grow | The Rust compiler uses `Vec<Token>`, `Vec<HFunction>`, `Vec<HStatement>` everywhere. Without growable arrays, you cannot build a token stream or AST. |
| Hash maps / associative arrays | Symbol tables (`HashMap<String, Option<Type>>`), struct field maps, scope lookups, function signature registry | High | Not available in Antimony | The Rust compiler uses `HashMap` in: symbol tables, struct field maps in QBE generator, `fn_signatures`, `fn_param_types`, `seen` import tracking. A compiler without key-value lookup is crippled. |
| Enums / tagged unions | Token kinds (30+ variants), AST node types (Statement has 9 variants, Expression has 11), Type enum, BinOp enum | High | Not in the language | This is the single biggest gap. The entire compiler is built on enums: `TokenKind`, `HStatement`, `HExpression`, `Type`, `BinOp`, `Keyword`. Without tagged unions + match, AST representation requires ugly struct-with-tag-field workarounds. |
| Heap allocation (malloc/free or equivalent) | AST nodes are heap-allocated (Box<HExpression>), strings are dynamic, all data structures grow at runtime | High | Not exposed in Antimony | Every `Box<HExpression>`, every string concatenation, every `Vec::push` requires heap allocation. The C builtins use malloc but Antimony code cannot call it directly. |
| Pointers / references | AST is a tree of pointers (`Box<HStatement>`, `Box<HExpression>`), recursive data structures require indirection | High | Not in the language | Without pointers, you cannot have recursive types (e.g., an Expression that contains sub-Expressions). This blocks AST representation entirely. |
| Struct methods with self | Parser methods (`self.next()`, `self.peek()`), Generator methods, Builder methods | Medium | Partially exists (`self` keyword in structs) | The `Selff` keyword exists in the AST. Need to verify QBE backend handles method dispatch correctly. |

### String & Character Features

| Feature | Why Required | Complexity | Current Status | Notes |
|---------|-------------|------------|----------------|-------|
| Character-level string access (indexing) | Lexer scans character by character (`self.bump()`, `self.first()`), token extraction, escape sequence handling | High | Not available | The Rust lexer calls `self.chars()`, indexes into strings, compares individual characters. Without `s[i]` or equivalent char access, you cannot write a lexer. |
| String concatenation | Error messages, code generation (building output string), string building in general | Low | Works (via `_str_concat` builtin) | Already functional. |
| String comparison | Token matching (`==` on strings), keyword identification, identifier comparison | Medium | Unclear for QBE | The parser compares token strings constantly. Need to verify string equality works in QBE target. |
| Character/byte type | Lexer needs to inspect individual characters, compare against `'a'`, `'\n'`, etc. | Medium | No char type exists | Types are: int, string, bool, any, array, struct. No char/byte type. Could workaround with int + ASCII values, but it's painful. |
| String slicing / substring | Token extraction (`raw.truncate(len)`), error reporting (`highlight_position_in_file`) | Medium | Not available | The Rust compiler uses `truncate`, string slicing, `collect::<String>()`. Need at minimum a `substring(s, start, end)` builtin. |
| Int-to-string conversion | Error messages with line numbers, position reporting, code generation of numeric literals | Low | Available (`_int_to_str` builtin) | Already present in QBE builtins. |

### File I/O Features

| Feature | Why Required | Complexity | Current Status | Notes |
|---------|-------------|------------|----------------|-------|
| Read file to string | Builder loads source files (`File::open`, `read_to_string`) | Medium | Not available in Antimony | The Builder opens files and reads them. Without file reading, you cannot load source files to compile. |
| Write string to file | Builder writes generated output to files | Medium | Not available in Antimony | Final compilation output must be written somewhere. |
| File path manipulation | Import resolution: relative paths from importing file, path joining | Medium | Not available | The Builder resolves imports relative to the importing file's directory. Needs path join, parent directory extraction. |
| Command-line argument access | Compiler needs input file path, output path, target selection | Low | Not available | `sb build input.sb -o output.ssa --target qbe` -- the self-hosted compiler needs CLI arg parsing. |

### Control Flow Features

| Feature | Why Required | Complexity | Current Status | Notes |
|---------|-------------|------------|----------------|-------|
| Match/switch on values | Parser dispatches on token kinds, generators dispatch on AST node types | Low | Match exists in language | Already in HAST, lowered to if-else chains. Works for simple values. |
| Recursion | Parser rules are recursive (parse_expression calls parse_primary calls parse_expression), AST traversal in generators is recursive | Low | Should work | Fundamental to any compiler. Verify stack depth is adequate for deeply nested AST. |
| Early return | Parser error handling returns early on failure, many functions short-circuit | Low | Works (`return` statement exists) | Already supported. |
| Break/continue in loops | Lexer eat_while loops, parser recovery | Low | Works | Already in the language. |

### Type System Features

| Feature | Why Required | Complexity | Current Status | Notes |
|---------|-------------|------------|----------------|-------|
| Optional/nullable values | Parser state (`current: Option<Token>`, `prev: Option<Token>`), function return types (`Option<Type>`) | Medium | Not in the language | The Rust compiler uses `Option<T>` pervasively. Without it, need sentinel values (null pointer, magic int, empty string). Workable but ugly. |
| Type annotations on all variables | QBE backend needs to know types for correct codegen; self-hosted compiler needs typed data structures | Low | Already exists | Variable declarations support `: type` annotations. |
| Struct nesting (struct fields that are structs) | AST nodes contain other AST nodes, Token contains Position, HFunction contains HVariable | Low | Works (sandbox.sb example shows nested structs) | Already demonstrated with Point/Rectangle example. |

## Differentiators

Features that make the compiler **nicer** to write but are not strictly required. These could be deferred or worked around.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Generics / type parameters | `Vec<Token>`, `HashMap<String, Type>` -- without generics, need separate array/map types for each element type, or use `any` everywhere | Very High | Not needed if using `any` type + runtime casting. Many early self-hosting compilers (Go 1.0, early C compilers) lacked generics. Use `any[]` arrays and untyped maps. |
| Error type / Result type | The Rust compiler uses `Result<T, String>` everywhere. Without it, error handling is return-code + global error string | Medium | Can work around with struct containing success bool + value + error string, or convention-based error returns. |
| Closures / first-class functions | Would make AST traversal elegant, but not required | Very High | Skip entirely. No self-hosting compiler needs closures. Use explicit function calls. |
| String interpolation / format strings | Error messages are built with `format!()` in Rust. Without it, lots of string concatenation | Low | Nice to have. Can always do `"Error at line " + int_to_str(line) + ": " + msg`. |
| Bitwise operations | Useful for efficient flag sets, hash functions. Not strictly required. | Low | Could be added as builtins relatively cheaply. |
| Negative integer literals | `-1` as sentinel values, array index arithmetic | Low | Currently `Int(usize)` -- unsigned only. Would need to verify if subtraction producing negative values works in QBE. |
| Multi-line strings / raw strings | Useful for embedding QBE output templates in the code generator | Low | Nice for the code generator but string concatenation works. |
| Module-level constants | Magic numbers (MAX_TOKENS, etc.) could be named | Low | Currently no `const` -- use functions returning values as workaround. |

## Anti-Features

Features to explicitly **not** build for the bootstrap milestone. These are over-engineering traps.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Full generics system | Months of work for type checking, monomorphization, QBE codegen. Go bootstrapped without generics for 10 years. | Use `any` type for polymorphic containers. Cast at usage sites. Accept the runtime unsafety for bootstrap. |
| Garbage collection | Compilers are short-lived processes. Memory leaks don't matter for a program that runs for milliseconds. | Just malloc, never free. Let the OS reclaim memory on exit. Every early bootstrap compiler does this. |
| Algebraic data types with full pattern matching | Real ADTs need type checking, exhaustiveness checking, destructuring codegen. Massive scope. | Use struct-with-integer-tag pattern: `struct Token { kind: int, raw: string, line: int }`. Match on the integer tag. Ugly but sufficient. |
| Trait/interface system | The Rust compiler's `Generator` trait is elegant but unnecessary when you know exactly which generator you're targeting (QBE). | Hardcode QBE generation. No dynamic dispatch needed. The self-hosted compiler only needs one backend. |
| Sophisticated error recovery | The Rust parser does error recovery with position tracking and source highlighting. This is UX polish, not correctness. | Crash on first error with line number. Fix the source file. Re-run. This is how every bootstrap compiler works. |
| Iterator/functional patterns | The Rust compiler uses `.filter()`, `.map()`, `.collect()` chains. Antimony doesn't need this. | Use explicit while loops. More verbose, equally correct. |
| Unicode support in lexer | The Rust lexer handles Unicode identifiers. The bootstrap compiler only needs to compile Antimony source which can be restricted to ASCII. | ASCII-only lexer. Antimony source files for the compiler itself will use ASCII identifiers only. |
| REPL or incremental compilation | Nice for development, irrelevant for bootstrap. | Batch compilation only. Read file, compile, write output. |
| Multiple backends in self-hosted compiler | The Rust compiler supports JS, C, QBE, x86, LLVM. The self-hosted compiler only needs QBE. | Single backend (QBE SSA output). Half the code, fraction of the complexity. |
| Optimization passes | The self-hosted compiler doesn't need to generate fast code -- just correct code. QBE handles optimization. | Naive codegen that produces correct QBE IR. Let QBE optimize. |

## Feature Dependencies

```
Heap Allocation ──> Dynamic Arrays (growable vectors)
                ──> Hash Maps
                ──> Recursive AST (Box pointers)

Pointers/References ──> Recursive Data Structures (AST)
                    ──> Dynamic Arrays (internal pointer to buffer)
                    ──> Hash Maps (internal pointer to buckets)

Character Access ──> Lexer Implementation
String Slicing   ──> Lexer Implementation (token extraction)
                 ──> Error Messages (source highlighting)

Enums OR Struct-with-Tag ──> Token Representation
                         ──> AST Node Representation
                         ──> Type Representation
                         ──> Operator Representation

Dynamic Arrays ──> Token Stream (lexer output)
               ──> AST Children (statement lists, argument lists)
               ──> Scope Stack (parser, generator)
               ──> Module Function/Struct Lists

Hash Maps ──> Symbol Table
          ──> Struct Field Lookup (generator)
          ──> Import Tracking (circular import detection)
          ──> Function Signature Registry (generator pre-pass)

File I/O ──> Source File Loading
         ──> Output File Writing
         ──> Import Resolution (read imported files)

CLI Args ──> Compiler Entry Point
```

## Critical Path Analysis

The dependencies reveal a clear critical path:

1. **Heap allocation + pointers** must come first -- everything else depends on them
2. **Dynamic arrays** next -- needed by every compiler phase
3. **Hash maps** (or simpler associative structure) -- needed for symbol tables
4. **Character-level string access** -- needed for the lexer
5. **Enum-or-workaround decision** -- needed for token/AST representation
6. **File I/O** -- needed to actually read source files
7. **CLI argument access** -- needed for the compiler entry point

## MVP Recommendation

Prioritize (in order):

1. **Heap allocation primitives** (malloc/free exposed as builtins) -- unlocks everything else
2. **Pointer type** in the language -- enables recursive data structures
3. **Dynamic array** (growable vector via stdlib, backed by malloc + realloc) -- enables token streams, AST node lists
4. **Character access on strings** (s[i] returning int/char, or a `char_at(s, i)` builtin) -- enables the lexer
5. **String slicing** (`substring(s, start, end)` builtin) -- enables token extraction
6. **Simple hash map** (stdlib implementation using arrays, or a builtin) -- enables symbol tables
7. **File read/write builtins** (`_read_file(path): string`, `_write_file(path, content)`) -- enables source loading
8. **CLI args builtin** (`_args(): string[]` or similar) -- enables compiler entry point

Defer:
- **Enums:** Use struct-with-integer-tag pattern instead. The Rust compiler's ~30 TokenKind variants become `let TOKEN_IF: int = 1`, `let TOKEN_ELSE: int = 2`, etc. Ugly but proven (this is how C compilers bootstrap).
- **Generics:** Use `any` type. Accept type unsafety for bootstrap.
- **Optional type:** Use null pointers (0) or sentinel values.
- **Error type:** Use convention (return 0 for success, or return a struct with an error field).
- **Garbage collection:** Never free. The compiler is short-lived.

## The Struct-with-Tag Pattern (Enum Workaround)

Since enums are the biggest missing feature, here's concretely how to work around it:

```
// Instead of enum TokenKind { If, Else, Int(value), Str(value), ... }
// Use integer constants + a struct:

fn TOKEN_IF(): int { return 1 }
fn TOKEN_ELSE(): int { return 2 }
fn TOKEN_INT(): int { return 3 }
fn TOKEN_STR(): int { return 4 }
fn TOKEN_IDENT(): int { return 5 }
// ... etc for all ~30 token kinds

struct Token {
    kind: int
    raw: string
    line: int
    offset: int
}

// Pattern match becomes if-else chain:
fn is_keyword(t: Token): bool {
    if t.kind == TOKEN_IF() {
        return true
    }
    if t.kind == TOKEN_ELSE() {
        return true
    }
    return false
}
```

This pattern applies to: TokenKind, Statement variants, Expression variants, Type variants, BinOp variants. It's verbose but requires zero language changes.

## Quantified Scope

Based on the Rust compiler's structure, here's what the self-hosted compiler must implement:

| Component | Rust LOC | Estimated Antimony LOC | Key Dependencies |
|-----------|----------|----------------------|------------------|
| Lexer (tokenization) | ~550 | ~400-600 | char access, string slicing, dynamic arrays |
| Parser (syntax analysis) | ~975 | ~800-1200 | token stream, AST structs, recursion |
| AST types | ~260 (HAST) | ~150-250 | structs, tag constants |
| AST transform | ~300 | ~200-350 | AST types, struct creation |
| Type inference | ~170 | ~150-250 | symbol table (hash map), AST traversal |
| Builder (orchestration) | ~175 | ~150-250 | file I/O, parser, import resolution |
| QBE generator | ~1750 | ~1500-2500 | AST traversal, string building, struct maps |
| CLI + main | ~50 | ~30-50 | CLI args, file I/O |
| Stdlib (builtins) | N/A (Rust std) | ~200-400 | New: vector, hashmap, string utils |
| **Total** | **~4230** | **~3600-5850** | |

The self-hosted compiler will likely be 4000-5000 lines of Antimony, assuming the struct-with-tag workaround for enums (which adds verbosity) and explicit while-loop iteration (vs Rust's iterators).

## Sources

- Direct analysis of the Antimony codebase at `/Users/garrit/src/garritfra/antimony/src/`
- AST types: `src/ast/hast.rs`, `src/ast/last.rs`, `src/ast/types.rs`
- Lexer: `src/lexer/mod.rs`, `src/lexer/cursor.rs`
- Parser: `src/parser/parser.rs`, `src/parser/rules.rs`
- QBE generator: `src/generator/qbe.rs` (1753 lines)
- Builtins: `builtin/builtin_qbe.c` (current QBE runtime primitives)
- Stdlib: `lib/*.sb` (current standard library)
- Confidence: HIGH -- findings derived from actual codebase analysis, not hypothetical requirements
