# Technology Stack

**Project:** Antimony QBE Backend Self-Hosting
**Researched:** 2026-03-23

## Executive Context

Antimony must mature its QBE backend until the compiler can be rewritten in Antimony and compiled via QBE. This document identifies the specific technology layers -- QBE IR features, C runtime bindings, language primitives, and stdlib functions -- required to reach that milestone, based on analysis of the current codebase and the known requirements of compiler-writing.

## Current State Assessment

### What Works Today in QBE Backend

| Feature | Status | QBE IR Used |
|---------|--------|-------------|
| Integer arithmetic | Working | `add`, `sub`, `mul`, `div`, `rem` (Word) |
| Boolean logic | Working | `and`, `or`, comparison instructions (Word) |
| String literals | Working | Data definitions with byte arrays, NUL-terminated |
| String concatenation | Working | Calls `_str_concat` C builtin |
| Function calls | Working | `call` instruction, System V ABI |
| If/else | Working | `jnz` + block labels |
| While loops | Working | Block labels + `jmp`/`jnz` |
| For-in loops | Working | Lowered to counter-based iteration over arrays |
| Break/continue | Working | Jump to loop end/condition labels |
| Structs | Working | `alloc8`, field offset calculation, `store`/`load` |
| Struct methods | Working | Name-mangled functions with `self` pointer |
| Arrays (int[]) | Working | Header (Long length) + contiguous elements, `alloc8` |
| Array access | Working | Pointer arithmetic with element size scaling |
| Type coercion (int->string) | Working | Calls `_int_to_str` builtin |
| Match statements | Working | Lowered to if/else chains in HAST->LAST transform |

### What Is Missing for Self-Hosting

| Missing Feature | Why Needed for Compiler | Severity |
|-----------------|------------------------|----------|
| File I/O | Read source files, write .ssa output | CRITICAL |
| Dynamic memory allocation | Hash maps, dynamic arrays, AST nodes | CRITICAL |
| Mutable string operations | Token/string building, substring, char-at | CRITICAL |
| Enums / tagged unions | Token kinds, AST node types, error variants | CRITICAL |
| Pointer types | Linked structures, nullable references | CRITICAL |
| Character type / byte access | Lexer character scanning | HIGH |
| String comparison | Token matching, keyword lookup | HIGH |
| String slicing / substring | Parsing identifiers, literals from source | HIGH |
| HashMap / associative array | Symbol tables, scope management | HIGH |
| Growable/dynamic arrays | Token lists, AST children, error accumulation | HIGH |
| Integer-to-string formatting | SSA output generation (register names, constants) | MEDIUM |
| Null/None handling | Optional values, uninitialized variables | MEDIUM |
| Bitwise operations | Potential for flag fields, enum discriminants | MEDIUM |
| Multi-return / tuples | Error handling patterns (Result-like) | LOW |

## Recommended Stack

### Core: QBE 1.2 (External Tool)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| QBE | 1.2 | SSA-to-assembly backend | Already in use. Stable release. Produces x86-64 and aarch64 assembly. Used by cproc (C compiler) and Hare language, proving it handles real compiler workloads. |
| GCC/Clang | System | Assembler + linker | QBE emits `.s` assembly; needs a system assembler/linker to produce binaries. GCC already used in pipeline. |

**Confidence:** HIGH -- QBE 1.2 is the version downloaded in CI. The tool is stable with infrequent releases.

### Core: qbe Rust Crate 3.0.0 (Build Dependency)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| qbe | 3.0.0 | Rust API for generating QBE IL | Already in use. Provides typed IR construction. Used until bootstrap replaces Rust compiler. |

**Confidence:** HIGH -- pinned in Cargo.toml.

**Note:** After bootstrap, the Rust crate is no longer needed. The self-hosted Antimony compiler will emit QBE IL as text (`.ssa` files) directly, same as cproc does in C.

### QBE IR Features Required

These are the QBE IL instructions and capabilities the backend must use. All are available in QBE 1.2.

#### Data Types

| QBE Type | Antimony Mapping | Current | Needed |
|----------|-----------------|---------|--------|
| `w` (Word, 32-bit) | `int`, `bool` | Used | Keep |
| `l` (Long, 64-bit) | Pointers, `string`, `any`, arrays | Used | Keep |
| `b` (Byte, 8-bit) | Character/byte access | Partially used (string data defs) | Expand: need byte loads/stores for lexer |
| `s` (Single, 32-bit float) | -- | Not used | Not needed for bootstrap |
| `d` (Double, 64-bit float) | -- | Not used | Not needed for bootstrap |
| Aggregate types (`:name`) | Structs, array layouts | Used | Keep |

**Confidence:** HIGH -- these are fundamental QBE types documented in the IL spec.

#### Instructions Needed but Not Yet Emitted

| Instruction | Purpose | Needed For |
|-------------|---------|------------|
| `storeb` | Store a byte to memory | Character-level string building |
| `loadub` / `loadsb` | Load a byte from memory | Lexer: reading characters from source string |
| `call $malloc` | Dynamic heap allocation | AST nodes, hash maps, growable buffers |
| `call $free` | Heap deallocation | Memory management (optional, can leak for compiler) |
| `call $fopen` / `$fread` / `$fwrite` / `$fclose` | File I/O | Reading source, writing output |
| `call $memcpy` | Bulk memory copy | Buffer operations, string building |
| `call $strcmp` / `$strncmp` | String comparison | Keyword matching, symbol lookup |

**Confidence:** HIGH -- these are standard C library functions callable from QBE via `call` with no special handling needed. QBE passes arguments per the System V AMD64 ABI.

#### Instructions Already Used (Confirmed in Codebase)

`add`, `sub`, `mul`, `div`, `rem`, `and`, `or`, `cmp` (all comparison variants), `copy`, `alloc8`, `store`, `load`, `call`, `jmp`, `jnz`, `ret`, `blit`, `extsw`, `extuw`, `extub`

### C Runtime Layer (builtin_qbe.c)

The current `builtin_qbe.c` provides 7 functions. Self-hosting requires expanding this significantly.

#### Current Builtins (Keep)

| Function | Signature | Purpose |
|----------|-----------|---------|
| `_printf` | `void(char*)` | Print string to stdout |
| `_exit` | `void(int)` | Exit process |
| `_int_to_str` | `char*(long)` | Format integer as string |
| `_str_concat` | `char*(char*, char*)` | Concatenate two strings |
| `_strlen` | `int(char*)` | String length |
| `_parse_int` | `int(char*)` | Parse decimal string to int |
| `_read_line` | `char*()` | Read line from stdin |

#### New Builtins Required

**File I/O (CRITICAL for reading source, writing output):**

| Function | Signature | Purpose | Wraps |
|----------|-----------|---------|-------|
| `_file_open` | `long(char*, char*)` | Open file, return handle | `fopen` |
| `_file_close` | `void(long)` | Close file handle | `fclose` |
| `_file_read_all` | `char*(char*)` | Read entire file to string | `fopen` + `fread` + `fclose` |
| `_file_write` | `void(long, char*)` | Write string to file handle | `fputs` or `fwrite` |
| `_file_write_all` | `void(char*, char*)` | Write string to path | Convenience wrapper |

**Confidence:** HIGH -- these are thin wrappers around POSIX/C89 file APIs. The existing builtin pattern (C functions with `_` prefix callable from Antimony) is proven.

**Memory Management (CRITICAL for dynamic data structures):**

| Function | Signature | Purpose | Wraps |
|----------|-----------|---------|-------|
| `_malloc` | `long(long)` | Allocate heap memory | `malloc` |
| `_realloc` | `long(long, long)` | Resize allocation | `realloc` |
| `_free` | `void(long)` | Free allocation | `free` |
| `_memcpy` | `void(long, long, long)` | Copy memory | `memcpy` |
| `_memset` | `void(long, int, long)` | Fill memory | `memset` |

**Confidence:** HIGH -- direct libc wrappers.

**String Operations (CRITICAL for lexer/parser):**

| Function | Signature | Purpose | Wraps |
|----------|-----------|---------|-------|
| `_str_char_at` | `int(char*, int)` | Get character at index | Array access |
| `_str_substr` | `char*(char*, int, int)` | Extract substring | `malloc` + `memcpy` |
| `_str_eq` | `int(char*, char*)` | String equality | `strcmp` == 0 |
| `_str_starts_with` | `int(char*, char*)` | Prefix check | `strncmp` |
| `_str_contains` | `int(char*, char*)` | Substring search | `strstr` |
| `_str_index_of` | `int(char*, char)` | Find character | `strchr` |
| `_str_from_char` | `char*(int)` | Single char to string | `malloc` + assign |

**Confidence:** HIGH -- straightforward C implementations.

**Process/OS (for compiler CLI):**

| Function | Signature | Purpose | Wraps |
|----------|-----------|---------|-------|
| `_system` | `int(char*)` | Run shell command (invoke QBE, GCC) | `system` |
| `_getenv` | `char*(char*)` | Read environment variable | `getenv` |

**Confidence:** HIGH -- one-line wrappers.

### Language Features Required

These are Antimony language-level features (parser + codegen) needed before the compiler can be self-hosted.

#### Tier 1: CRITICAL (Cannot Write Compiler Without These)

**Enums / Tagged Unions**

The current Rust compiler represents tokens as `enum TokenKind` with ~40 variants and AST nodes as `enum Statement`/`Expression` with ~15 variants each. The Antimony self-hosted compiler needs equivalent discriminated unions.

Recommended approach: Implement as integer-tagged structs. An enum value is a struct with a `tag: int` discriminant field plus a payload. Match statements already lower to if/else chains, so `match` on enum tag values works.

QBE impact: No new IR needed. Enums are structs with an int tag field. The existing struct codegen handles this.

**Confidence:** MEDIUM -- this is a language design decision. The "tagged struct" approach is what cproc and many QBE-targeting languages use, but Antimony's specific syntax needs design work.

**Pointer / Reference Types**

The compiler needs nullable pointers for optional AST children (else branches, return types), linked list nodes, and tree structures. Currently, Antimony has no pointer type.

Recommended approach: Add a `ptr` type (or `T?` for nullable) that maps to QBE `l` (Long). Pointer dereference and address-of operators.

QBE impact: Pointers are just Long values in QBE. No new IR instructions needed -- `load` and `store` already work with Long pointers.

**Confidence:** MEDIUM -- language design decision. QBE handles this trivially.

**Dynamic/Growable Arrays**

Token lists, AST node children, and error lists grow during compilation. Fixed-size `alloc8` arrays are insufficient.

Recommended approach: Implement as a stdlib data structure using `_malloc`/`_realloc` builtins. A `Vec`-like struct: `{ data: ptr, len: int, cap: int }`.

QBE impact: No new IR. Uses existing struct + call + pointer arithmetic.

**Confidence:** HIGH -- standard approach for any compiled language.

**HashMap / Dictionary**

Symbol tables are `HashMap<String, Type>` in the Rust compiler. The self-hosted compiler needs equivalent functionality.

Recommended approach: Implement as an Antimony stdlib module. Open-addressing hash table using arrays and string hashing. Requires: string hashing builtin (`_str_hash` in C), dynamic arrays.

QBE impact: No new IR. Pure library code built on arrays + structs + builtins.

**Confidence:** HIGH -- well-understood data structure.

#### Tier 2: HIGH (Needed for Practical Compiler Writing)

**Character / Byte Type**

The lexer scans source character-by-character. Antimony needs a way to work with individual characters (bytes).

Recommended approach: Either add a `char` type mapping to QBE `w` (Word) or use `int` for character values with `_str_char_at` builtin. The latter is simpler and avoids a new type.

QBE impact: Minimal. `loadub` for byte access, which QBE already supports.

**Confidence:** HIGH.

**String Comparison in Conditionals**

`if token == "fn"` must work. Currently string equality is not implemented in QBE backend binary operations.

Recommended approach: Detect string `==` in BinOp generation (same pattern as string `+` already uses `_str_concat`) and emit `call _str_eq`.

QBE impact: None. Just a codegen pattern change to call the C builtin.

**Confidence:** HIGH -- follows the exact pattern already used for string concatenation.

**Error Reporting with Source Positions**

The compiler needs to report errors with file/line/column. Requires tracking source positions through the lexer.

Recommended approach: Structs with `line: int, col: int, file: string` fields. Already representable with current struct support.

QBE impact: None.

**Confidence:** HIGH.

#### Tier 3: MEDIUM (Quality of Life)

**Multiple Return Values / Simple Error Handling**

The Rust compiler uses `Result<T, String>` extensively. Without generics, the self-hosted compiler needs a simpler pattern.

Recommended approach: Use a struct like `{ ok: ptr, err: string }` or use a global error flag pattern. Many bootstrapped compilers use global error state.

QBE impact: None.

**Confidence:** MEDIUM -- design trade-off.

**Bitwise Operations**

Useful for hash functions, flag fields, enum discriminant packing.

Recommended approach: Add `&`, `|`, `^`, `<<`, `>>` operators. QBE has `and`, `or`, `xor`, `shl`, `shr` instructions already.

QBE impact: Already available in QBE IL. Need lexer + parser + codegen wiring.

**Confidence:** HIGH for QBE support; MEDIUM for language integration.

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Backend | QBE 1.2 | LLVM | Project constraint. QBE is the explicit target. Also: LLVM is 100x the complexity for this use case. |
| Runtime glue | C builtins (builtin_qbe.c) | Raw syscalls | C builtins are simpler, portable, and the existing pattern works. Raw syscalls are architecture-specific and fragile. |
| Enum implementation | Tagged structs (int discriminant) | Separate type in IR | QBE has no sum types. Tagged structs are how cproc and Hare implement them. Proven pattern. |
| HashMap | Open-addressing in stdlib | External C library (e.g. uthash) | Pure Antimony implementation proves the language is capable. External C dependency defeats self-hosting spirit. |
| String handling | C builtins for low-level ops | Pure QBE string manipulation | String operations in pure QBE IR would be verbose and error-prone. C builtins are 5-10 lines each and battle-tested. |
| Memory management | Manual malloc/free via builtins | Garbage collector | GC is massive scope creep. Manual memory management is fine for a compiler (predictable allocation patterns). Leaking is acceptable for a single-run tool. |
| Dynamic arrays | Stdlib Vec struct | Language-level growable arrays | Library approach requires no language changes. Can be built entirely with existing struct + pointer + builtin support. |

## Compilation Pipeline (Self-Hosted)

The bootstrap pipeline will be:

```
Phase 0 (now):     .sb  -->  [Rust compiler]  -->  .ssa  -->  [QBE 1.2]  -->  .s  -->  [GCC]  -->  binary
Phase 1 (bootstrap): .sb  -->  [Antimony compiler (compiled by Phase 0)]  -->  .ssa  -->  [QBE 1.2]  -->  .s  -->  [GCC]  -->  binary
Phase 2 (verify):  Antimony compiler source  -->  [Phase 1 binary]  -->  .ssa  -->  [QBE]  -->  .s  -->  [GCC]  -->  binary2
                   Verify: binary == binary2 (or equivalent output)
```

The self-hosted compiler emits `.ssa` text directly (no Rust `qbe` crate involved). This means:

- The compiler needs string formatting to build QBE IL text
- `_int_to_str` (already exists) handles numeric temporaries
- String concatenation (already exists) builds instruction lines
- File write (new builtin) outputs the `.ssa` file
- `_system` (new builtin) invokes `qbe` and `gcc` for full compilation

## Installation / Build Changes

```bash
# No new external tools needed beyond what CI already installs:
# QBE 1.2 (already in CI)
curl -fsSL https://c9x.me/compile/release/qbe-1.2.tar.xz | tar xJ -C /tmp
cd /tmp/qbe-1.2 && make && sudo cp qbe /usr/local/bin/

# GCC (already required by QBE pipeline)
# No new Rust crate dependencies needed

# New: builtin_qbe.c will grow from ~55 lines to ~200-300 lines
# Still compiled and linked as part of the QBE build pipeline
```

## Reference: Languages Successfully Self-Hosted via QBE

**cproc** -- C11 compiler written in C, uses QBE as backend. Self-hosting since ~2020. Demonstrates that QBE handles the full complexity of a real compiler. cproc emits QBE IL as text strings directly, which is the same approach Antimony's self-hosted compiler should use.

**Hare** -- Systems programming language using QBE backend. Has a complete standard library including file I/O, memory allocation, string handling. Demonstrates that a QBE-targeting language can provide a rich enough stdlib for systems programming.

**Confidence:** MEDIUM -- based on training data. Could not verify current status via web search (permission denied), but both projects are well-known in the QBE ecosystem.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| QBE IR features | HIGH | Verified from qbe.rs codebase -- all instructions already represented in the Rust `qbe` crate |
| C runtime builtins needed | HIGH | Standard libc functions; existing builtin pattern proven |
| Language features needed | MEDIUM | Design decisions for enums/pointers not yet made; QBE support is certain but Antimony syntax is open |
| Self-hosting pipeline | HIGH | Follows proven cproc/Hare pattern |
| Version numbers | HIGH | QBE 1.2 from CI config, qbe crate 3.0.0 from Cargo.toml |
| Hare/cproc reference | MEDIUM | Training data only; could not web-verify current status |

## Gaps to Address

- **Enum syntax design**: How will tagged unions be expressed in Antimony? Match arms need to destructure variants. This is a language design question, not a QBE limitation.
- **Pointer syntax design**: `ptr`, `T?`, `*T`? The QBE side is trivial (Long values), but the language syntax affects ergonomics.
- **Memory management strategy**: Full manual malloc/free, or arena allocation, or leak-everything for compiler? Affects stdlib design.
- **Error handling pattern**: Without generics, how does the self-hosted compiler report and propagate errors?
- **QBE variadic calls**: `printf` with format strings needs QBE's `...` variadic calling convention. The `qbe` Rust crate supports this but Antimony's codegen may not emit it correctly. Verify before relying on direct `printf` calls (current `_printf` wrapper avoids this issue).

## Sources

- `/Users/garrit/src/garritfra/antimony/src/generator/qbe.rs` -- Full QBE codegen implementation (1753 lines)
- `/Users/garrit/src/garritfra/antimony/builtin/builtin_qbe.c` -- Current C runtime (55 lines, 7 functions)
- `/Users/garrit/src/garritfra/antimony/src/ast/types.rs` -- Current type system (Int, Str, Bool, Array, Struct, Any)
- `/Users/garrit/src/garritfra/antimony/src/ast/last.rs` -- Low-level AST (what codegen sees)
- `/Users/garrit/src/garritfra/antimony/.github/workflows/ci.yml` -- QBE 1.2 version confirmed
- `/Users/garrit/src/garritfra/antimony/Cargo.toml` -- qbe crate 3.0.0 confirmed
- QBE IL specification at c9x.me/compile/doc/il.html (could not fetch; referenced from training data)
- cproc and Hare as QBE-targeting self-hosted languages (training data references)
