# Domain Pitfalls

**Domain:** Compiler bootstrapping via QBE backend maturation
**Researched:** 2026-03-23
**Overall confidence:** HIGH (grounded in codebase analysis and established compiler engineering knowledge)

## Critical Pitfalls

Mistakes that cause rewrites or months of lost effort.

### Pitfall 1: The Moving-Target Bootstrap -- Changing the Language While Writing the Compiler In It

**What goes wrong:** You add a language feature to make writing the self-hosted compiler easier, but that feature change invalidates compiler code you already wrote in Antimony. You end up in a loop: the language evolves, the self-hosted compiler breaks, you fix it, realize you need another feature, and the cycle repeats indefinitely.

**Why it happens:** Self-hosting creates a circular dependency. The language defines what the compiler can express. The compiler defines what programs can run. Changing either side destabilizes the other.

**Consequences:** The bootstrap never converges. Every "small" language tweak cascades through hundreds of lines of self-hosted compiler code. The project stalls in perpetual iteration.

**Prevention:**
1. **Freeze the language before starting the rewrite.** Complete the gap audit, add all required features (enums, pointers, file I/O, string ops), then declare a feature freeze. The self-hosted compiler targets this frozen spec.
2. **If a missing feature is discovered mid-rewrite, implement it in the Rust compiler first, write a test, then continue.** Do not redesign -- add the minimum viable version.
3. **Keep a "language debt" list** of things you wish were different. Address them after the bootstrap succeeds, not during.

**Detection:** You find yourself modifying `src/parser/` or `src/ast/` while simultaneously writing the Antimony rewrite. That is the warning sign.

**Phase relevance:** The gap audit phase must be thorough enough that the language freeze is credible. If you discover missing features during the rewrite, the audit was incomplete.

---

### Pitfall 2: The C Builtin Trap -- Building a Second Runtime Instead of Language Primitives

**What goes wrong:** Every time the QBE backend needs a capability (string concatenation, int-to-string conversion, array allocation), you add another C function to `builtin_qbe.c`. The "Antimony language" becomes a thin wrapper around a growing C runtime. When it comes time to bootstrap, the self-hosted compiler depends on a C runtime it cannot express or replace.

**Why it happens:** It is the path of least resistance. Need `strlen`? Call libc. Need `malloc`? Call libc. The `builtin_qbe.c` file already has this pattern (`_printf`, `_exit`, `_int_to_str`, `_str_concat`, `_strlen`, `_parse_int`, `_read_line`). Every new builtin is easy.

**Consequences:** The self-hosted compiler inherits a hard dependency on a C runtime that was never designed as a coherent API. The bootstrap is not really "Antimony compiled by Antimony" -- it is "Antimony compiled by Antimony plus an unknown amount of C glue." More critically: you cannot write the replacement for `builtin_qbe.c` in Antimony because Antimony lacks the primitives to do so (no pointer types, no raw memory access).

**Prevention:**
1. **Draw a clear line** between "syscall wrappers that will always be C/assembly" (write, read, mmap, exit) and "runtime functions that should eventually be expressible in Antimony" (string concat, int-to-string, array operations).
2. **Keep the C layer minimal and stable.** Syscall wrappers only. Anything above syscall level should be implementable in Antimony.
3. **Add pointer types to the language** so that Antimony code can do its own memory management via syscall wrappers, rather than calling `malloc` through C.

**Detection:** Count the functions in `builtin_qbe.c`. If it grows beyond ~10, you are building a runtime, not a language. Currently at 7 -- this is near the danger zone.

**Phase relevance:** Must be addressed during the "runtime primitives" phase, before the rewrite begins.

---

### Pitfall 3: No Enums/Sum Types Means No AST Representation

**What goes wrong:** You try to write the self-hosted parser and immediately discover you cannot represent an AST. The Rust compiler uses `enum Statement { If { ... }, While { ... }, Return(...), ... }` and `enum Expression { Int(usize), Str(String), BinOp { ... }, ... }` as its core data structures. Antimony has no enums. Without them, the compiler literally cannot be written.

**Why it happens:** Enums are a "nice to have" for application code but are load-bearing for compiler internals. The gap audit might deprioritize them because existing test programs do not need them. But the compiler itself is not a typical program.

**Consequences:** This is a hard blocker for self-hosting. There is no workaround that does not result in deeply unergonomic code. You could simulate enums with integer tags and structs, but that eliminates exhaustive checking and is a maintenance nightmare for a program as large as a compiler.

**Prevention:**
1. **Implement enums/sum types as a first-class language feature before the rewrite.** This is not optional.
2. **Design them to support associated data** (tagged unions), not just C-style enumerations. The AST requires `Expression::BinOp { lhs, op, rhs }`, not just `Expression = 0 | 1 | 2`.
3. **Test with a miniature AST** (e.g., a calculator expression tree) to validate that the enum implementation handles recursive types, pattern matching, and exhaustiveness.

**Detection:** If you start the rewrite and find yourself encoding node types as integers with manual dispatch, you skipped this step.

**Phase relevance:** Must be implemented in the gap-closing phase. Block the rewrite on this.

---

### Pitfall 4: String Handling That Works for Hello World But Not for Compilers

**What goes wrong:** Antimony's current string model works for printing messages. A compiler needs: character-by-character iteration (the lexer), substring extraction, string comparison, string building with append, character-to-integer conversion (for parsing digits), and handling strings that contain null bytes or special characters. None of these exist.

**Why it happens:** The current string implementation is backed by C's null-terminated `char*` via the QBE builtin layer. This works for `println("hello")` but is insufficient for a lexer that needs to examine individual characters, track positions, and build tokens character by character.

**Consequences:** The lexer is the first thing you write in a self-hosted compiler. If you cannot iterate over characters in a string, you are blocked on page one. Even if you add character access, null-terminated strings mean you cannot have strings containing `\0`, which limits what source files can contain.

**Prevention:**
1. **Implement character access** (`s[i]` or a `char_at` function) as a language primitive or stdlib function backed by a minimal C builtin.
2. **Implement substring/slice operations** (`s[start..end]` or equivalent).
3. **Implement string comparison** (`==` for strings) in the QBE backend. Currently `==` on strings likely compares pointers, not content.
4. **Consider length-prefixed strings** instead of null-terminated, so the representation is self-describing. This aligns with how arrays already work (8-byte length header).
5. **Implement string building** (a growable buffer or repeated append) efficiently -- the current `_str_concat` allocates a new buffer every time, which is O(n^2) for building a string character by character.

**Detection:** Try to write a function in Antimony that counts the occurrences of character `'a'` in a string. If you cannot, strings are not ready.

**Phase relevance:** Must be resolved in the "runtime primitives" phase. The lexer depends on it.

---

### Pitfall 5: Type System Too Weak to Type-Check Itself

**What goes wrong:** Antimony's type system has `Int`, `Str`, `Bool`, `Array(Type)`, `Struct(name)`, and `Any`. To write a compiler, you also need: enums (Pitfall 3), generic/parameterized types (e.g., `Option<T>`, `Result<T, E>` or equivalents), HashMap/dictionary types, and recursive types (an expression contains sub-expressions). If the type system cannot express the compiler's own data structures, the rewrite is blocked.

**Why it happens:** The type system was designed for simple programs. A compiler is not a simple program -- it is one of the most type-intensive programs you can write.

**Consequences:** You either block on missing type features or resort to `Any` everywhere, which defeats the purpose of having types and makes the self-hosted compiler undebuggable.

**Prevention:**
1. **Map every Rust type used in the current compiler** to what Antimony would need. For each one, decide: implement it, work around it, or change the compiler design to avoid it.
2. **At minimum, you need:** enums with data, structs (already have), arrays (already have), some form of hash map (for symbol tables), and `Option`-like nullable types.
3. **You do not need full generics** for the bootstrap. Concrete types for each use case (e.g., `StringMap` rather than `HashMap<String, T>`) are sufficient.

**Detection:** Try to write the type definitions for Token, Statement, Expression, and SymbolTable in Antimony. If any of them requires a type that does not exist, that is the gap.

**Phase relevance:** Language feature phase. Must precede the rewrite.

---

### Pitfall 6: Unsafe Transmute in QBE Codegen Hides Undefined Behavior

**What goes wrong:** The QBE generator currently uses `std::mem::transmute` (lines 201-205, 1632-1634) to cast `qbe::Type<'_>` to `qbe::Type<'static>` in order to work around lifetime constraints in the `qbe` crate. This is undefined behavior if the referenced data (`TypeDef`) is ever moved or dropped while the transmuted reference is still live. As the codegen grows more complex (more struct types, nested structs, arrays of structs), the chance of triggering this UB increases.

**Why it happens:** The `qbe` Rust crate (v3.0.0) uses lifetimes for `Type::Aggregate` that make it difficult to store aggregate types alongside the module being built. The transmute is a workaround for an API friction.

**Consequences:** Subtle codegen bugs: wrong struct layouts, corrupted field offsets, segfaults in compiled programs that only appear with certain struct nesting patterns. These are nightmares to debug because the source of the bug is in the Rust compiler, not in the generated QBE IL.

**Prevention:**
1. **Pin the `TypeDef` allocations** so they cannot move. The current `Rc<TypeDef>` approach in `typedefs: Vec<RcTypeDef>` is a step in the right direction but does not fully prevent issues.
2. **Long-term: fork or contribute to the `qbe` crate** to eliminate the lifetime constraint on `Type::Aggregate`. The TODO on line 24 acknowledges this.
3. **Add integration tests that compile programs with nested structs, arrays of structs, and structs containing strings.** If these pass under both debug and release builds, the transmute is likely safe in practice.

**Detection:** Run the full test suite under Miri or AddressSanitizer. If the transmute is unsound, Miri will catch it.

**Phase relevance:** Should be addressed in early stabilization, before the codegen gets more complex.

---

## Moderate Pitfalls

### Pitfall 7: No File I/O Means the Compiler Cannot Read Source Files

**What goes wrong:** A compiler reads source files and writes output files. Antimony's QBE stdlib has `print`, `println`, `read_line`, and `exit`. There is no `read_file`, `write_file`, or any filesystem access. The self-hosted compiler cannot load `.sb` source files.

**Prevention:**
1. Add C builtins for `_read_file(path) -> string` and `_write_file(path, content)` as syscall-level wrappers.
2. Expose them through the stdlib as `read_file` and `write_file`.
3. Keep them minimal -- the bootstrap only needs "read entire file to string" and "write string to file." Seeking, appending, and streaming can wait.

**Detection:** Can you write a program in Antimony that reads a `.sb` file and prints its contents? If not, file I/O is missing.

**Phase relevance:** Runtime primitives phase.

---

### Pitfall 8: Memory Leaks Accumulate in Long-Running Compilation

**What goes wrong:** Every `_str_concat` call in `builtin_qbe.c` allocates with `malloc` and never frees. Every `_int_to_str` allocates and never frees. For a "hello world" program this does not matter. For a compiler processing thousands of tokens and building large ASTs, memory usage grows without bound.

**Prevention:**
1. **For the bootstrap, accept the leaks.** A compiler is a batch program -- it runs, produces output, and exits. The OS reclaims all memory. This is the approach GCC took for decades.
2. **Do not try to build a garbage collector before bootstrapping.** That is a multi-month distraction.
3. **If memory becomes a practical problem** (compiler OOMs on large files), implement arena allocation: allocate a large block, sub-allocate from it, free the entire block at the end.

**Detection:** Compile a large Antimony program (1000+ lines) and watch memory usage with `top`. If it exceeds 100MB, leaks are becoming practical.

**Phase relevance:** Post-bootstrap optimization. Do not let this block the rewrite.

---

### Pitfall 9: QBE's Integer Model vs. Antimony's `int` Type

**What goes wrong:** Antimony has a single `int` type. QBE has `Word` (32-bit) and `Long` (64-bit). The QBE generator currently maps `Int` to `Word` (32-bit). This means: array indices are 32-bit (limiting arrays to 4GB), file sizes are limited to 2GB, and pointer arithmetic wraps at 32 bits. For a compiler processing normal source files, this is fine. But it is a latent bug that will surface if Antimony is ever used for programs handling large data.

**Prevention:**
1. **For the bootstrap, 32-bit int is fine.** Source files are not 2GB.
2. **Document the decision.** "Antimony `int` is 32-bit signed. Pointers are 64-bit (Long). Array lengths are stored as Long but truncated to Word when returned from `len()`."
3. **When pointer types are added, make them 64-bit (Long)** regardless of `int` width. The current codegen already does this for struct pointers and array base pointers.

**Detection:** Array indexing beyond 2^31 elements would fail silently. Not a practical concern for the bootstrap.

**Phase relevance:** Design decision to document during gap audit. Revisit post-bootstrap.

---

### Pitfall 10: Testing the QBE Backend by Generating SSA Instead of Running Binaries

**What goes wrong:** The existing QBE unit tests (1834+ lines in `qbe_tests.rs`) verify the generated QBE IL text. They check that the right instructions are emitted. They do not compile the IL through QBE, assemble it, link it, and run it. This means a test can pass even if the generated IL produces wrong results when actually executed.

**Prevention:**
1. **Add end-to-end execution tests** for the QBE target: compile `.sb` file to binary via the full pipeline (`.sb -> .ssa -> .s -> binary`), run the binary, check exit code and stdout.
2. **Run these tests in CI.** They require QBE and GCC to be installed.
3. **Mirror every unit test with an execution test.** The unit test verifies the IL shape; the execution test verifies correctness.

**Detection:** A QBE unit test passes but the corresponding example program produces wrong output when compiled and run. This has likely already happened but gone unnoticed.

**Phase relevance:** Set up the execution test harness in the first phase. Run it continuously.

---

### Pitfall 11: Name Mangling Collisions in Method Dispatch

**What goes wrong:** Methods are mangled as `StructName_methodName` (single underscore). If a struct named `Foo` has method `bar`, it becomes `Foo_bar`. If there is also a top-level function named `Foo_bar`, the names collide. Since the self-hosted compiler will have many structs with common method names (`parse`, `next`, `emit`, `generate`), collisions become likely.

**Prevention:**
1. **Use a double underscore or other unambiguous separator** for mangled names (e.g., `Foo__bar`).
2. **Add a check during the pre-pass** that detects name collisions between mangled method names and top-level function names.

**Detection:** Linker error about duplicate symbol definitions, or worse, silent wrong dispatch.

**Phase relevance:** Fix before the rewrite, when only a few structs exist.

---

## Minor Pitfalls

### Pitfall 12: Type Inference in QBE Generator Instead of Parser

**What goes wrong:** The QBE generator has its own return-type inference logic (`infer_fn_return_type`, lines 74-153) that duplicates and diverges from the parser's inference pass (`src/parser/infer.rs`). The TODO on line 71 acknowledges this. As the language grows, these two inference implementations will disagree, causing QBE-only bugs.

**Prevention:** Consolidate type inference into the parser. All generators should receive fully-typed ASTs.

**Phase relevance:** Stabilization phase, before adding new types.

---

### Pitfall 13: The "Just One More Feature" Scope Creep Before Bootstrap

**What goes wrong:** The gap audit reveals 20 missing features. You start implementing them. Each one reveals two more. Months pass. The bootstrap never starts.

**Prevention:**
1. **Define the minimal subset of the compiler that constitutes a valid bootstrap.** It does not need to support every backend -- just QBE. It does not need perfect error messages. It needs to correctly compile itself.
2. **Prioritize features by whether the compiler source code actually needs them**, not by what would be "nice to have" in the language.
3. **Start writing the self-hosted compiler early**, even in parallel with gap-closing. The code you write reveals which gaps actually matter.

**Detection:** More than 3 months of gap-closing without a single line of self-hosted compiler code written.

**Phase relevance:** Planning phase. Set a time-box for gap-closing.

---

### Pitfall 14: Forgetting That the Bootstrap Compiler Must Reproduce Itself

**What goes wrong:** You write a self-hosted compiler that produces binaries. Great. But can the self-hosted compiler compile its own source code and produce a binary that is identical (or functionally identical) to itself? If not, the bootstrap is not complete -- you just have a compiler written in Antimony, not a self-hosting compiler.

**Prevention:**
1. **The bootstrap test is:** Compile the self-hosted compiler with Rust-Antimony. Call the result `sb1`. Compile the self-hosted compiler source with `sb1`. Call the result `sb2`. Run `sb2` on the test suite. If tests pass, bootstrap is validated.
2. **Binary reproducibility** (sb1 == sb2 byte-for-byte) is ideal but not required. Functional equivalence (same behavior on all tests) is sufficient.
3. **Plan for this test from the beginning.** It is the acceptance criterion for the milestone.

**Detection:** You declare "bootstrap complete" without running the self-compilation round-trip.

**Phase relevance:** Acceptance criteria for the final phase.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Gap audit | Underestimating what a compiler needs (Pitfalls 3, 4, 5) | Write the Token/AST type defs in Antimony as a litmus test |
| Runtime primitives | Growing `builtin_qbe.c` into a second runtime (Pitfall 2) | Draw the syscall/library line before adding anything |
| Language features (enums, pointers) | Moving-target bootstrap (Pitfall 1) | Freeze language spec before rewrite starts |
| Stabilization | Tests pass on IL but fail on execution (Pitfall 10) | Set up end-to-end test harness first |
| Self-hosted rewrite | Scope creep (Pitfall 13) | Time-box gap-closing; start writing compiler code early |
| Bootstrap validation | Forgetting the round-trip test (Pitfall 14) | Define acceptance criteria before starting |

## Sources

- Codebase analysis of `/Users/garrit/src/garritfra/antimony/` (primary source for all findings)
- `src/generator/qbe.rs`: QBE codegen, transmute usage, type inference duplication, name mangling
- `src/ast/types.rs`: Type enum showing current type system limitations
- `src/ast/last.rs`: LAST definitions showing what the self-hosted compiler must represent
- `builtin/builtin_qbe.c`: C runtime functions, memory leak patterns
- `lib/`: Standard library showing current string/IO/array capabilities
- `src/parser/infer.rs`: Inference pass showing duplication with QBE generator
- Compiler bootstrapping knowledge: established patterns from GCC, Go, Rust, and Zig bootstrapping histories (MEDIUM confidence -- from training data, not verified against current sources)
