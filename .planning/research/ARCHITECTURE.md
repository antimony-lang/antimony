# Architecture Patterns

**Domain:** Self-hosting compiler bootstrap (Antimony-in-Antimony via QBE backend)
**Researched:** 2026-03-23

## Recommended Architecture

### The Three-Stage Bootstrap Model

Compiler bootstrapping follows a well-established pattern used by GCC, Go, Rust, and essentially every self-hosting compiler. The model has three stages:

```
Stage 0 (seed compiler):  Rust compiler (existing) compiles Antimony source
Stage 1 (first bootstrap): Stage 0 compiles the new Antimony-in-Antimony compiler
Stage 2 (verification):    Stage 1 compiler compiles itself; output must match Stage 1
```

**Stage 0** is the existing Rust compiler (`sb`). It already works. It compiles `.sb` files to QBE SSA, which feeds through `qbe` and `gcc` to produce a native binary.

**Stage 1** is the first real test: can the Rust compiler produce a working Antimony compiler binary from the new Antimony source code? If `sb build compiler.sb -o compiler.ssa --target qbe` produces a working compiler, Stage 1 passes.

**Stage 2** is the correctness proof: the Stage 1 binary compiles its own source code. If the Stage 2 output binary is identical (or functionally identical) to Stage 1, the bootstrap is verified. Bit-for-bit reproducibility is the gold standard but not strictly required for a first bootstrap.

### Source Tree Structure

The new compiler source should live alongside the Rust compiler, not replace it. The Rust compiler must remain buildable throughout the entire process because it is Stage 0.

```
antimony/
  src/                          # Existing Rust compiler (Stage 0) - DO NOT MODIFY
    lexer/
    parser/
    ast/
    builder/
    generator/
    command/
    main.rs

  sb/                           # New Antimony-in-Antimony compiler source
    lexer.sb                    # Token definitions, tokenize()
    parser.sb                   # Token stream -> AST
    ast.sb                      # AST node definitions (structs)
    types.sb                    # Type enum and type-related logic
    transform.sb                # HAST -> LAST lowering
    builder.sb                  # Module resolution, import handling
    generator_qbe.sb            # QBE SSA code generation (only backend needed)
    main.sb                     # CLI entry point, orchestration

  lib/                          # Shared standard library (.sb files)
  builtin/                      # Shared C runtime (builtin_qbe.c)

  Makefile                      # Bootstrap build rules (see Build Order below)
```

Key decisions in this layout:

- **`sb/` directory, not replacing `src/`**. The Rust compiler is the seed and must always work. Never touch `src/` during the bootstrap effort.
- **Single QBE backend only.** The self-hosted compiler does not need JS, C, x86, or LLVM generators. It only needs to emit QBE SSA. This dramatically reduces scope.
- **Flat module structure.** Antimony's module system uses file-based imports. A flat `sb/` directory with one file per compiler phase keeps imports simple and avoids deep nesting that complicates the build.
- **Shared `lib/` and `builtin/`**. The standard library and C runtime builtins are target-agnostic. Both compilers use them.

### Component Boundaries

| Component | Responsibility | Communicates With | Antimony File |
|-----------|---------------|-------------------|---------------|
| **Lexer** | Source string -> token array | Parser consumes tokens | `sb/lexer.sb` |
| **AST Types** | Token, AST node, Type struct/enum definitions | Used by all components | `sb/ast.sb`, `sb/types.sb` |
| **Parser** | Token array -> AST (high-level) | Lexer output, AST types | `sb/parser.sb` |
| **Type Inference** | Annotate AST nodes with types | Parser output, AST types | `sb/parser.sb` (inline) |
| **Transform** | High-level AST -> low-level AST | Parser output, AST types | `sb/transform.sb` |
| **Builder** | File I/O, module resolution, orchestration | Lexer, Parser, Transform, Generator | `sb/builder.sb` |
| **QBE Generator** | Low-level AST -> QBE SSA string | AST types, builtin knowledge | `sb/generator_qbe.sb` |
| **Main** | CLI args, entry point | Builder | `sb/main.sb` |

### Data Flow

The data flow mirrors the existing Rust compiler exactly. This is intentional -- the Antimony compiler is a direct port, not a redesign.

```
Source file (.sb)
    |
    v
Lexer: tokenize(source_string) -> Token[]
    |
    v
Parser: parse(tokens) -> HModule (high-level AST)
    |
    v
Type Inference: infer(hmodule) -> HModule (annotated)
    |
    v
[For each import: recursively Lexer -> Parser -> Infer]
    |
    v
Module Merge: merge all HModules + stdlib into one HModule
    |
    v
Transform: lower(hmodule) -> Module (low-level AST)
    |
    v
QBE Generator: generate(module) -> String (QBE SSA text)
    |
    v
Write .ssa file to disk
    |
    v
External: qbe .ssa -> .s, gcc .s -> binary
```

## Build Order: Which Components to Implement First

This is the critical architectural question. The build order is driven by **testability** -- each component should be independently testable as soon as it is written.

### Phase 1: Data Structures and Lexer

**Why first:** The lexer is the simplest compiler phase. It requires only string operations, character classification, and struct construction. It exercises the language features most likely to already work in QBE: integers, strings, structs, arrays, while loops, function calls.

1. **Type definitions** (`sb/types.sb`): Define the Type enum equivalent. Antimony does not have enums yet, so this must use tagged structs (integer tag + fields). This is the first design decision the bootstrap forces.
2. **Token/AST definitions** (`sb/ast.sb`): Token, TokenKind, Keyword, Value as tagged structs. Statement/Expression node types.
3. **Lexer** (`sb/lexer.sb`): Character-by-character scanning, token construction, keyword recognition.

**Testability:** Compile `lexer.sb` with the Rust compiler targeting QBE. Feed it a small `.sb` file. Print tokens. Compare against Rust compiler output.

### Phase 2: Parser

**Why second:** The parser is the most complex component by code volume, but it follows a mechanical recursive-descent pattern. Every parse function follows the same shape: peek token, match, consume, build AST node.

4. **Parser** (`sb/parser.sb`): Recursive descent. Functions for each grammar production. Builds HModule with HFunction/HStatement/HExpression nodes.
5. **Type inference** (inline in parser or separate): Symbol table construction, type propagation.

**Testability:** Parse a `.sb` file, walk the AST, print a summary. Compare against expected structure.

### Phase 3: Transform and Generator

**Why third:** The transform is a straightforward tree-to-tree rewrite. The QBE generator is where Antimony's code meets QBE's expectations -- but the Rust implementation already defines exactly what QBE SSA to emit for each AST node.

6. **Transform** (`sb/transform.sb`): HAST -> LAST lowering. Desugar match to if-else chains, for-in to while loops.
7. **QBE Generator** (`sb/generator_qbe.sb`): Walk the LAST, emit QBE SSA text. This is a direct port of `src/generator/qbe.rs`.

**Testability:** Full pipeline test. Compile a `.sb` file through the Antimony compiler, diff the `.ssa` output against what the Rust compiler produces.

### Phase 4: Builder and Main

**Why last:** The builder requires file I/O and path manipulation -- the most OS-dependent functionality. The main entry point requires argument parsing.

8. **Builder** (`sb/builder.sb`): File reading, import resolution, module merging, stdlib inclusion.
9. **Main** (`sb/main.sb`): Argument parsing (minimal: input file, output file), orchestrate build pipeline.

**Testability:** The full compiler. `stage0_sb build sb/main.sb -o compiler.ssa --target qbe && qbe compiler.ssa -o compiler.s && gcc compiler.s builtin/builtin_qbe.c -o sb_stage1`. Then `./sb_stage1 build sb/main.sb -o compiler2.ssa` and diff.

## Patterns to Follow

### Pattern 1: Tagged Structs Instead of Enums

**What:** Antimony does not have enum/sum types. The Rust compiler uses enums extensively (TokenKind, Statement, Expression, Type, BinOp). The Antimony compiler must simulate these with tagged structs: an integer tag field plus optional payload fields.

**When:** Every place the Rust compiler uses an enum.

**Example (conceptual Antimony):**
```
// Instead of Rust's:  enum TokenKind { Plus, Minus, Identifier(String), ... }
// Use:
struct TokenKind {
    tag: int           // 0=Plus, 1=Minus, 2=Identifier, ...
    str_value: string  // populated when tag=Identifier or tag=StringLiteral
}

fn is_plus(tk: TokenKind): bool {
    return tk.tag == 0
}
```

**Critical implication:** This pattern requires careful discipline. Every `match` on a Rust enum becomes a chain of `if tk.tag == N` checks. Define constants for each tag value. The lack of exhaustiveness checking means bugs from missed cases are likely -- test heavily.

### Pattern 2: Array-Based Collections Instead of HashMap

**What:** The Rust compiler uses HashMap for symbol tables, struct metadata, and function signatures. Antimony likely does not have a HashMap. Use sorted arrays with linear or binary search instead.

**When:** Any place the Rust compiler uses HashMap.

**Example (conceptual Antimony):**
```
struct SymbolEntry {
    name: string
    ty: int  // type tag
}

// Linear search for small tables, binary search if performance matters
fn lookup_symbol(table: SymbolEntry[], name: string): int {
    let i: int = 0
    while i < len(table) {
        if table[i].name == name {
            return i
        }
        i += 1
    }
    return -1  // not found
}
```

**Why acceptable:** Compiler symbol tables for typical Antimony programs are small (hundreds of entries, not millions). O(n) lookup on small tables is fine. A HashMap would require implementing hash functions, bucket management, and resizing -- unnecessary complexity.

### Pattern 3: Direct Port, Not Redesign

**What:** The Antimony compiler should be a line-by-line port of the Rust compiler's logic, adapted only where Antimony's type system forces it (no enums, no generics, no traits).

**Why:** The goal is bootstrap, not a better compiler. The Rust implementation is tested, working, and well-understood. Redesigning during a bootstrap attempt adds risk with zero benefit. Once the bootstrap works, the self-hosted compiler can be refactored in Antimony.

**When:** Always. Resist the temptation to "improve" during porting.

### Pattern 4: Bootstrapping Makefile

**What:** A Makefile that encodes the stage0/stage1/stage2 pipeline.

**Example:**
```makefile
STAGE0 = cargo run --
QBE = qbe
CC = gcc
BUILTINS = builtin/builtin_qbe.c

# Stage 1: Rust compiler builds the Antimony compiler
stage1: sb/*.sb
	$(STAGE0) build sb/main.sb -o build/stage1.ssa --target qbe
	$(QBE) build/stage1.ssa -o build/stage1.s
	$(CC) build/stage1.s $(BUILTINS) -o build/stage1

# Stage 2: Antimony compiler builds itself
stage2: stage1 sb/*.sb
	./build/stage1 build sb/main.sb -o build/stage2.ssa
	$(QBE) build/stage2.ssa -o build/stage2.s
	$(CC) build/stage2.s $(BUILTINS) -o build/stage2

# Verify: Stage 1 and Stage 2 produce identical SSA
verify: stage2
	diff build/stage1.ssa build/stage2.ssa
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Modifying the Rust Compiler During Bootstrap

**What:** Changing `src/` to accommodate the bootstrap effort.
**Why bad:** The Rust compiler is Stage 0. If it breaks, the entire bootstrap chain breaks. Any "temporary" changes to help bootstrap tend to become permanent technical debt.
**Instead:** If the Rust compiler has bugs that block the bootstrap, fix them in separate commits before the bootstrap work begins. The QBE gap audit already planned in PROJECT.md is exactly the right approach.

### Anti-Pattern 2: Trying to Bootstrap a Language Subset

**What:** Writing the compiler in a restricted subset of Antimony to avoid implementing hard features.
**Why bad:** The PROJECT.md explicitly rejects this: "Partial bootstrap doesn't prove the language is capable; full rewrite is the real milestone." A subset bootstrap leaves the hardest problems unsolved and creates a false sense of completion.
**Instead:** If a language feature is needed for the compiler but not yet working in QBE, implement it in the QBE backend first (via the gap audit), then use it in the Antimony compiler.

### Anti-Pattern 3: Building Multiple Backends in the Self-Hosted Compiler

**What:** Porting all five generators (JS, C, QBE, x86, LLVM) to Antimony.
**Why bad:** Massive scope increase for zero bootstrap benefit. Only QBE is needed.
**Instead:** The self-hosted compiler emits QBE SSA only. Other backends can be added later if ever needed.

### Anti-Pattern 4: Implementing Data Structures From Scratch First

**What:** Building a full standard library (HashMap, dynamic arrays, string builder, etc.) before starting the compiler.
**Why bad:** You will over-engineer data structures for imagined needs. Let the compiler's actual needs drive what gets built.
**Instead:** Start with the simplest possible implementations (arrays, linear search). Only upgrade when the compiler demonstrably needs it -- which it probably never will, given the scale.

## How the Rust and Antimony Compilers Coexist

The two compilers coexist permanently. The Rust compiler never goes away.

**During development:**
- The Rust compiler (`cargo build`) is always the authoritative Stage 0
- Each new Antimony compiler component is tested by compiling it with Stage 0
- The Antimony compiler source (`sb/`) is just another Antimony program from Stage 0's perspective

**After bootstrap is achieved:**
- Stage 0 (Rust) remains in the repo as the seed compiler
- Day-to-day development could use either compiler
- The Rust compiler serves as the reference implementation for correctness
- New language features must be implemented in both compilers (or the Rust compiler remains frozen as a bootstrap seed)

**Long-term options (decide later, not now):**
1. **Freeze Rust compiler:** Stage 0 is frozen at the bootstrap point. All future development happens in the Antimony compiler. The Rust compiler is only used to bootstrap from scratch.
2. **Maintain both:** Both compilers evolve. This is expensive but provides a cross-check.
3. **Drop Rust compiler:** Once bootstrap is well-established, remove the Rust compiler. This is what Go did (dropped C implementation). Only do this when confidence in the Antimony compiler is very high.

Option 1 is the standard approach and the recommended one for this project.

## Scalability Considerations

These are irrelevant for a bootstrap compiler. The compiler only needs to compile programs the size of itself (thousands of lines, not millions). Performance optimization is an anti-pattern at this stage. Get it correct first.

| Concern | At Bootstrap | Post-Bootstrap |
|---------|-------------|----------------|
| Compile speed | Irrelevant -- correctness is the goal | Optimize if compiling large programs |
| Memory usage | Likely fine -- compiler source is small | Profile if compiling 100k+ line programs |
| Error messages | Minimal but useful position info | Improve iteratively |
| Symbol table perf | Linear search is fine | Consider hash tables for large programs |

## Sources

- Codebase analysis: `src/generator/qbe.rs`, `src/ast/last.rs`, `src/lexer/mod.rs`, `src/builder/mod.rs`, `src/generator/mod.rs`, `builtin/builtin_qbe.c`
- Project context: `.planning/PROJECT.md`, `.planning/codebase/ARCHITECTURE.md`
- Compiler bootstrapping is a well-established discipline. The stage0/stage1/stage2 model is documented in the histories of GCC, Go, Rust, and numerous other self-hosting compilers. Confidence: HIGH (stable domain knowledge, unchanged for decades).
