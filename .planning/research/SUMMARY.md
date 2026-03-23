# Project Research Summary

**Project:** Antimony QBE Backend Self-Hosting
**Domain:** Compiler bootstrapping — Antimony compiler rewritten in Antimony, compiled via QBE backend
**Researched:** 2026-03-23
**Confidence:** HIGH

## Executive Summary

Antimony's path to self-hosting is a classic three-stage bootstrap: the existing Rust compiler (Stage 0) compiles a new Antimony-in-Antimony compiler (Stage 1), which then compiles itself (Stage 2) to verify correctness. This is the exact pattern used by GCC, Go, and Rust, and it has been proven viable for QBE-targeting languages by both cproc (a self-hosting C11 compiler) and the Hare language. The QBE 1.2 backend is already stable in CI and handles everything a compiler needs at the IR level — the blockers are entirely at the language and C-runtime layer, not in QBE itself.

The recommended approach is: audit the language gaps, close them in a deliberate sequence (heap allocation and pointers first, then dynamic arrays and string primitives, then file I/O), declare a language freeze, then write the Antimony compiler as a direct port of the existing Rust implementation rather than a redesign. The self-hosted compiler needs only a single QBE backend, no generics, no GC, and no trait system — the scope is deliberately narrow to make bootstrap achievable. Enums-as-tagged-structs, arrays-with-linear-search instead of hash maps, and malloc-then-never-free are all proven patterns from historic bootstrap efforts and are explicitly recommended here over more ambitious alternatives.

The key risk is the moving-target trap: if language features are added reactively during the rewrite phase, the self-hosted compiler code breaks repeatedly and the project never converges. Prevention requires completing the gap audit thoroughly before writing a single line of the self-hosted compiler, then holding the language spec stable for the duration of the rewrite. A secondary risk is the C builtin trap — letting `builtin_qbe.c` grow into a de-facto runtime rather than staying as thin syscall wrappers. The current count of 7 builtins is already near the danger zone, and the planned additions (file I/O, memory management, string ops) must be designed as the permanent, minimal API boundary between Antimony and C.

## Key Findings

### Recommended Stack

The stack is simple and already in place. QBE 1.2 (confirmed in CI) is the sole compilation backend; no alternatives were considered because QBE is an explicit project constraint and is proven sufficient by cproc's self-hosting. The `qbe` Rust crate (v3.0.0, pinned in Cargo.toml) is used by the Rust compiler but disappears at bootstrap: the self-hosted compiler will emit QBE SSA as plain text files, exactly as cproc does. GCC remains the assembler and linker in the pipeline.

**Core technologies:**
- **QBE 1.2:** SSA-to-native-assembly backend — stable, proven, already in CI pipeline
- **qbe Rust crate 3.0.0:** Typed IR construction in Rust compiler — present until bootstrap, then irrelevant
- **GCC (system):** Assembler and linker — QBE emits `.s` assembly; GCC produces the final binary
- **builtin_qbe.c (expanded):** C runtime bridge — current 7 functions grow to ~25; provides file I/O, malloc, string ops, and process control as the minimal boundary between Antimony and the OS

All required QBE IR instructions (`storeb`, `loadub`, `call $malloc`, `call $fopen`, `call $strcmp`, etc.) are standard QBE 1.2 features callable via the System V AMD64 ABI. No QBE patches or version changes are needed.

### Expected Features

The features research mapped every Rust compiler component to what Antimony must provide, producing a clear prioritized list. The critical path runs through heap allocation and pointer types — without those, dynamic arrays and hash maps cannot be built, and without those, the compiler cannot be written.

**Must have (table stakes — bootstrap is impossible without these):**
- Heap allocation primitives (`_malloc`/`_realloc` builtins) — unlocks all dynamic data structures
- Pointer/reference type — enables recursive data structures and dynamic collection internals
- Growable dynamic arrays (stdlib Vec-like struct) — needed for token streams, AST node lists, scope stacks
- Character-level string access (`char_at` or `s[i]`) — lexer scans character by character
- String slicing/substring — token extraction from source string
- String equality comparison (`==` on strings via `_str_eq`) — token matching and keyword lookup
- File read/write builtins (`_file_read_all`, `_file_write_all`) — source file loading and output writing
- CLI argument access — compiler entry point needs input/output file paths

**Should have (strongly recommended, partial workarounds exist):**
- Simple associative lookup (array-based linear search as HashMap substitute) — symbol tables, struct field maps
- Tagged struct pattern for enum simulation — token kinds, AST node types, operator types
- String comparison operators — parser dispatches on token strings constantly
- Character/byte type or integer-as-char convention — lexer character classification

**Defer to post-bootstrap:**
- Enums as first-class language feature — tagged struct workaround is sufficient for bootstrap
- Generics — use `any` type; accept type unsafety for bootstrap
- Garbage collection — never free; OS reclaims memory on process exit
- Multiple backends — self-hosted compiler only needs QBE output
- Error recovery — crash on first error; fix and re-run is acceptable for bootstrap
- Iterator/functional patterns — use explicit while loops

The self-hosted compiler is estimated at 4000-5000 lines of Antimony across 8 components, with the QBE generator (~1500-2500 lines) and parser (~800-1200 lines) being the dominant components.

### Architecture Approach

The architecture follows the canonical three-stage bootstrap model with no deviations. The new Antimony compiler source lives in a new `sb/` directory alongside the existing `src/` Rust compiler — the Rust compiler is never modified during the bootstrap effort, as it is the Stage 0 seed. The self-hosted compiler is a direct port of the Rust compiler's logic, not a redesign, because redesigning during bootstrap adds risk with zero benefit.

**Major components:**
1. **Lexer** (`sb/lexer.sb`) — source string to token array; first component to implement, tests string primitives
2. **AST Types** (`sb/ast.sb`, `sb/types.sb`) — Token, Statement, Expression, Type as tagged structs
3. **Parser** (`sb/parser.sb`) — recursive descent, token array to high-level AST; most complex by LOC
4. **Transform** (`sb/transform.sb`) — high-level AST to low-level AST; desugars match, for-in
5. **Builder** (`sb/builder.sb`) — file I/O, import resolution, module merging, stdlib inclusion
6. **QBE Generator** (`sb/generator_qbe.sb`) — direct port of `src/generator/qbe.rs`; emits SSA as text
7. **Main** (`sb/main.sb`) — CLI argument parsing, pipeline orchestration

Data flow is identical to the Rust compiler: source file -> Lexer -> Parser -> Type Inference -> Module Merge -> Transform -> QBE Generator -> write `.ssa` -> invoke `qbe` and `gcc`.

### Critical Pitfalls

1. **The moving-target bootstrap** — adding language features mid-rewrite breaks existing Antimony compiler code repeatedly; prevent by completing the gap audit, implementing all required features, then declaring a language freeze before writing the first line of the self-hosted compiler.

2. **The C builtin trap** — letting `builtin_qbe.c` grow into a de-facto runtime; prevent by drawing a hard line: only syscall-level wrappers in C, everything above that must be expressible in Antimony; current 7 functions are already near the limit.

3. **No enums means no AST representation** — the entire compiler is built on discriminated unions; prevent by implementing first-class enums before starting the rewrite, OR by fully committing to the tagged-struct workaround with integer constants before writing compiler code.

4. **String handling insufficient for a lexer** — character access, substring, and string equality are absent; prevent by implementing these as stdlib functions backed by minimal C builtins before the lexer is written; test with a character-counting function first.

5. **Unsafe transmute in QBE codegen** — `std::mem::transmute` in `qbe.rs` lines 201-205 and 1632-1634 is undefined behavior that can produce corrupted struct layouts in complex programs; prevent by adding integration tests for nested structs and running under Miri before codegen complexity increases.

## Implications for Roadmap

Based on research, the critical-path dependencies impose a clear phase order. Heap allocation and pointers must precede everything else; language features must be complete before the rewrite starts; the rewrite must produce a verifiable round-trip before the milestone is declared complete.

### Phase 1: QBE Backend Stabilization and Audit

**Rationale:** Before adding anything, establish a reliable baseline. The transmute UB (Pitfall 6), the duplicated type inference (Pitfall 12), and the name mangling collision risk (Pitfall 11) are existing technical debt that will compound during bootstrap work. Set up end-to-end execution tests now (Pitfall 10), because all subsequent phases depend on trusting the QBE backend. Do the gap audit — write Token, Statement, and Expression type definitions in Antimony to identify which gaps are genuinely blocking.
**Delivers:** Trusted QBE backend with execution tests; complete gap inventory; known-good Stage 0
**Addresses:** String comparison correctness, struct nesting correctness, method dispatch correctness
**Avoids:** Discovering UB-triggered codegen bugs mid-bootstrap (Pitfall 6); tests passing on IL but failing on execution (Pitfall 10)

### Phase 2: Runtime Primitives — Heap, Pointers, and String Operations

**Rationale:** Everything else depends on these. Heap allocation and pointer types unlock dynamic arrays, hash maps, and recursive data structures. String character access and substring enable the lexer. These must be complete before the language feature phase because higher-level features (dynamic arrays, hash maps) are implemented using them.
**Delivers:** `_malloc`, `_realloc`, `_free`, `_memcpy` builtins; pointer type in language; `_str_char_at`, `_str_substr`, `_str_eq`, `_str_starts_with` builtins; string comparison in QBE codegen's binary operator handling
**Uses:** Existing builtin pattern in `builtin_qbe.c`; existing struct + pointer codegen in QBE
**Avoids:** C builtin trap — define the syscall/library line explicitly before adding functions (Pitfall 2); string handling gap blocking the lexer (Pitfall 4)

### Phase 3: Standard Library — Dynamic Arrays and File I/O

**Rationale:** With heap allocation and pointers working, build the two most critical data structures: growable arrays and file access. Dynamic arrays are needed by every compiler phase. File I/O is needed by the builder. Both can be implemented as stdlib Antimony code backed by the new builtins from Phase 2. Linear-search arrays can substitute for hash maps in the bootstrap — implement those here as well.
**Delivers:** `Vec`-like growable array stdlib struct; `_file_read_all`, `_file_write_all`, `_file_open`, `_file_close` builtins; `_system`, `_getenv` process builtins; linear-search associative array stdlib struct; CLI argument access builtin
**Implements:** Architecture components: stdlib foundation that supports all compiler data structures
**Avoids:** Building data structures from scratch before verifying actual compiler needs (Anti-Pattern 4 from ARCHITECTURE.md)

### Phase 4: Language Feature Freeze — Enums or Committed Tagged-Struct Convention

**Rationale:** This is a decision gate. Either implement first-class enums (full-quality solution for AST representation) or formally commit to and document the tagged-struct workaround with integer constants for all variant types. Research recommends making this decision explicitly rather than drifting into the workaround. After this phase, the language spec is frozen. No new features are added until bootstrap is complete.
**Delivers:** Either: enum type in language with match pattern support; OR: documented tagged-struct convention with integer constants, tested with a full Token/Statement/Expression definition. Bitwise operators wired through to QBE `and`/`or`/`xor`/`shl`/`shr`. Language spec document.
**Avoids:** Moving-target bootstrap (Pitfall 1) — the freeze is the primary prevention mechanism; type system too weak for AST representation (Pitfall 5)

### Phase 5: Self-Hosted Compiler — Lexer and AST Types

**Rationale:** Start with the simplest compiler component. The lexer exercises string primitives and produces a concrete testable artifact (token stream) that can be compared against Rust compiler output. Building the lexer first forces any remaining string primitive gaps to surface early, before the parser is written.
**Delivers:** `sb/ast.sb` and `sb/types.sb` — all Token, Statement, Expression, Type definitions; `sb/lexer.sb` — tokenizes Antimony source; integration test comparing token output against Rust compiler
**Uses:** Character access, string comparison, dynamic arrays from Phases 2-3
**Avoids:** Scope creep (Pitfall 13) — time-box this phase; start writing before gap-closing feels "complete"

### Phase 6: Self-Hosted Compiler — Parser and Type Inference

**Rationale:** The parser is the largest component by LOC and the most mechanical to port. Recursive-descent parsers follow a regular pattern: peek, match, consume, recurse. The parser depends on the token types and dynamic arrays from Phase 5. Type inference is embedded in or immediately follows the parser.
**Delivers:** `sb/parser.sb` — recursive descent parser producing high-level AST; symbol table via linear-search associative arrays; type annotation propagation; integration test parsing sample programs
**Implements:** Parser and Type Inference architecture components

### Phase 7: Self-Hosted Compiler — Transform and QBE Generator

**Rationale:** The transform is a straightforward tree-to-tree rewrite that desugars high-level constructs. The QBE generator is the most lines of code but is a direct mechanical port of `src/generator/qbe.rs` — the Rust implementation already defines exactly what SSA to emit for every AST node. The generator produces SSA text directly (no `qbe` Rust crate involved), using string concatenation and `_int_to_str`.
**Delivers:** `sb/transform.sb` — HAST to LAST lowering; `sb/generator_qbe.sb` — walks LAST, emits QBE SSA text; diff test of SSA output against Rust compiler on sample programs
**Uses:** QBE 1.2 instruction set; string concatenation and `_int_to_str` for SSA text building

### Phase 8: Self-Hosted Compiler — Builder, Main, and Stage 1 Bootstrap

**Rationale:** The builder requires file I/O (available from Phase 3) and import resolution (path manipulation). The main entry point needs CLI argument parsing. Together these complete the compiler. The phase succeeds when the Rust compiler can compile `sb/main.sb` into a working binary that itself can compile Antimony programs.
**Delivers:** `sb/builder.sb` — file loading, import resolution, module merging; `sb/main.sb` — CLI entry point; Stage 1 binary (`build/stage1`) produced by Rust compiler; Stage 1 successfully compiles at least one test program
**Avoids:** Forgetting the round-trip test (Pitfall 14)

### Phase 9: Bootstrap Verification — Stage 2 Round-Trip

**Rationale:** Bootstrap is not complete until the Stage 1 compiler compiles its own source and produces a Stage 2 binary that passes the test suite. This is the acceptance criterion for the milestone. Functional equivalence (Stage 2 passes all tests Stage 1 passes) is sufficient; byte-for-byte reproducibility is a stretch goal.
**Delivers:** Stage 2 binary produced by Stage 1 compiler; Stage 2 passes full test suite; bootstrap milestone declared complete; Makefile with `stage1`, `stage2`, `verify` targets
**Avoids:** Declaring bootstrap complete without the round-trip test (Pitfall 14)

### Phase Ordering Rationale

- Phases 1-4 are pure prerequisite work: audit, runtime primitives, stdlib, and language freeze. The rewrite cannot start until all four complete.
- The language freeze (Phase 4) is a hard gate. Starting Phase 5 before the freeze guarantees hitting the moving-target pitfall.
- Phases 5-8 follow the natural data-flow dependency of the compiler itself: types/lexer before parser, parser before transform/generator, all components before the builder that orchestrates them.
- Phase 9 is a validation phase, not a build phase. Its content is predetermined by Phase 8's output.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 4 (Enum/tagged-struct decision):** Language design with no clear precedent in the Antimony codebase; the trade-offs between first-class enums and tagged-struct workaround need explicit evaluation against implementation cost
- **Phase 7 (QBE Generator port):** The Rust generator is 1753 lines with complex struct layout tracking and name mangling; porting details need a careful line-by-line plan; especially the struct typedef handling that currently uses transmute (Pitfall 6)
- **Phase 2 (Pointer type design):** Syntax (`ptr`, `T?`, `*T`) and semantics (null handling, dereference operators) are unresolved language design questions

Phases with standard patterns (research unlikely needed):
- **Phase 3 (File I/O and dynamic arrays):** Thin libc wrappers and a standard Vec-style struct; well-documented patterns
- **Phase 6 (Parser):** Recursive-descent parsing is a well-documented technique; mechanical port of existing implementation
- **Phase 9 (Bootstrap verification):** Binary diff or test-suite equivalence check; established bootstrap validation pattern

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | QBE 1.2 confirmed in CI config; qbe crate 3.0.0 confirmed in Cargo.toml; all required IR instructions verified in qbe.rs |
| Features | HIGH | Derived from direct codebase analysis of the Rust compiler; requirements are not hypothetical |
| Architecture | HIGH | Three-stage bootstrap is well-established domain knowledge; source tree structure follows natural constraints of the existing codebase |
| Pitfalls | HIGH | Critical pitfalls grounded in codebase analysis (transmute in qbe.rs, missing string ops in stdlib, 7 existing builtins); bootstrap pitfalls from well-documented compiler engineering history |

**Overall confidence:** HIGH

### Gaps to Address

- **Enum syntax design:** How will tagged unions be expressed? Match arms need to destructure variants. This is a language design question with no existing Antimony precedent. Resolve in Phase 4 planning with a concrete syntax proposal tested against a miniature AST.
- **Pointer type syntax:** `ptr`, `T?`, `*T` — each has different ergonomics for the nullable references needed throughout the parser. The QBE implementation is trivial (Long values) but the language syntax affects all compiler code. Resolve in Phase 2 planning.
- **Memory management strategy for bootstrap:** Full malloc/free, arena allocation, or leak-everything. Research recommends leak-everything for bootstrap (standard approach), but arena allocation is a viable upgrade if the compiler OOMs on large files.
- **Error handling pattern without generics:** The Rust compiler uses `Result<T, String>` pervasively. The struct-with-error-field pattern or global error state are the alternatives. Decide before the parser is written (Phase 6) since it affects every parsing function.
- **QBE variadic call convention:** Direct `printf` calls need QBE's `...` variadic syntax; the current `_printf` wrapper avoids this, but if the self-hosted compiler's generator needs to emit variadic QBE calls, this needs verification. Likely not an issue since the generator emits plain SSA text, not function calls to printf.

## Sources

### Primary (HIGH confidence)
- `/Users/garrit/src/garritfra/antimony/src/generator/qbe.rs` — QBE codegen implementation (1753 lines); transmute usage, name mangling, struct layout
- `/Users/garrit/src/garritfra/antimony/builtin/builtin_qbe.c` — Current C runtime (7 functions, 55 lines)
- `/Users/garrit/src/garritfra/antimony/src/ast/hast.rs`, `last.rs`, `types.rs` — Current type system and AST definitions
- `/Users/garrit/src/garritfra/antimony/src/lexer/mod.rs`, `cursor.rs` — Lexer implementation
- `/Users/garrit/src/garritfra/antimony/src/parser/parser.rs`, `rules.rs`, `infer.rs` — Parser and type inference
- `/Users/garrit/src/garritfra/antimony/.github/workflows/ci.yml` — QBE 1.2 version confirmed
- `/Users/garrit/src/garritfra/antimony/Cargo.toml` — qbe crate 3.0.0 confirmed

### Secondary (MEDIUM confidence)
- cproc (C11 compiler via QBE, self-hosting since ~2020) — demonstrates QBE handles full compiler workloads; emits QBE IL as text strings
- Hare language (QBE-targeting, complete stdlib) — demonstrates QBE-targeting language can provide systems-level stdlib
- GCC, Go, Rust bootstrap histories — source of three-stage bootstrap model and anti-patterns

### Tertiary (LOW confidence)
- QBE IL specification at c9x.me/compile/doc/il.html — referenced from training data; could not fetch directly
- Current status of cproc and Hare projects — training data only; could not web-verify

---
*Research completed: 2026-03-23*
*Ready for roadmap: yes*
