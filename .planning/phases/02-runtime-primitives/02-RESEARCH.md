# Phase 2: Runtime Primitives - Research

**Researched:** 2026-04-01
**Domain:** QBE backend runtime capabilities -- string ops, file I/O, CLI args, heap allocation, type inference fixes
**Confidence:** HIGH

## Summary

Phase 2 adds runtime primitives (string operations, file I/O, CLI argument access, heap allocation) to the QBE backend, and fixes type inference gaps that block the bootstrap path. The codebase has a well-established pattern for adding new builtins: QBE IL wrappers in `RUNTIME_PREAMBLE` for thin libc wrappers, C implementations in `builtin_qbe.c` for operations needing `malloc`/complex logic, and Antimony stdlib wrappers in `lib/*.sb`.

Critically, 4 of the 5 "bootstrap-blocking bugs" from the Phase 1 gap inventory (Bool codegen, And/Or operators, Str type inference, self.field assignment) are already fixed by commit b6e32be. The remaining bug -- method return type inference (`let v = obj.method()` fails without explicit type annotation) -- is confirmed still broken and is the primary inference fix needed. The `infer_builtin()` function in `infer.rs` also only knows about `len`, though this is less critical because most builtins have Antimony stdlib wrappers with return type annotations.

**Primary recommendation:** Focus effort on (1) the method return type inference fix in `infer.rs`, (2) new runtime builtins following the established RUNTIME_PREAMBLE + builtin_qbe.c + lib/*.sb pattern, and (3) argc/argv support which requires changing the QBE `main` function signature.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** File I/O implemented as thin QBE IL preamble wrappers around C stdlib (`fopen`, `fgets`/`fread`, `fwrite`, `fclose`). Same pattern as existing `_printf`, `_exit`, `_strlen` in `RUNTIME_PREAMBLE`. Exposed as low-level Antimony builtins. An `io` Antimony stdlib wrapper is deferred to Phase 3 or later.
- **D-02:** `malloc(size: int)` pointer exposed as a builtin. No `free`. This is an escape hatch -- the intended abstraction for most Antimony code is Phase 3 stdlib (dynamic arrays, string builder), which will call `malloc` internally. Exposing it now unblocks the bootstrap if Phase 3 stdlib isn't complete yet.
- **D-03:** Fix type inference broadly, not minimally: all builtins get return types at registration (not just `len`), method calls consult struct method return types properly, `self.field = expr` parser bug fixed as a targeted patch (not a systemic change). Goal: prevent the same class of inference failure from surfacing again in Phase 4/5 bootstrap work.
- **D-04:** Two builtins: `argc()` -> int and `argv(i: int)` -> str. Direct mapping to C's `main(int argc, char** argv)`. Safe -- avoids depending on string-array codegen.
- Test programs are self-checking: print PASS/FAIL, exit 0/1
- One .sb test file per feature in `tests/qbe/`
- New builtins follow the same registration pattern as existing ones

### Claude's Discretion
- Exact C stdlib functions chosen for file I/O wrappers (fopen/fgets vs fopen/fread -- whichever handles text files cleanly)
- Internal structure of builtin registration for return types
- Order of bug fixes vs new feature implementation within the phase

### Deferred Ideas (OUT OF SCOPE)
- `args()` -> str[] convenience builtin -- depends on string-array codegen being solid
- `io` Antimony stdlib module -- wraps file I/O builtins into higher-level interface, deferred to Phase 3+
- `free` / memory management -- explicitly out of scope; bootstrap compilers are batch programs that can leak
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| RUNTIME-01 | String character access works in QBE (index into string by position) | New C builtin `_str_char_at(s, i)` in `builtin_qbe.c` returning char as int; Antimony wrapper `str_char_at(s: string, i: int): int` in `lib/string.sb` |
| RUNTIME-02 | String comparison works in QBE (`==` on strings calls `strcmp`-equivalent) | New QBE IL preamble wrapper `_strcmp` around libc `strcmp`; QBE codegen must detect string `==`/`!=` and emit `_strcmp` call instead of `ceqw` |
| RUNTIME-03 | Substring extraction works in QBE | New C builtin `_str_substr(s, start, len)` in `builtin_qbe.c` using `malloc`+`memcpy`; Antimony wrapper in `lib/string.sb` |
| RUNTIME-04 | File I/O primitives available in Antimony (open, read, write, close) | QBE IL preamble wrappers `_fopen`, `_fclose` + C builtins `_fread_all`, `_fwrite_str` in `builtin_qbe.c`; Antimony wrappers in new `lib/file.sb` or `lib/io.sb` |
| RUNTIME-05 | CLI arguments accessible from Antimony programs (argc/argv) | QBE `main` signature change to `export function w $main(w %argc, l %argv)`; global stash pattern for argc/argv; builtins `argc()` and `argv(i)` |
| RUNTIME-06 | Heap allocation strategy decided and implemented | `_malloc` QBE IL preamble wrapper around libc `malloc`; Antimony builtin `malloc(size: int): int` returning raw pointer as int; no `free` |
</phase_requirements>

## Architecture Patterns

### Current Builtin Registration Pattern (3-layer)

Every QBE builtin follows a consistent 3-layer pattern:

```
Layer 1: QBE IL preamble (RUNTIME_PREAMBLE in src/generator/qbe.rs)
   OR    C implementation (builtin/builtin_qbe.c)

Layer 2: Return type registration (infer_builtin() in src/parser/infer.rs)

Layer 3: Antimony stdlib wrapper (lib/*.sb)
```

**Layer 1 decision criteria:**
- Simple libc forwarding (no malloc, no complex logic) -> QBE IL in RUNTIME_PREAMBLE
- Needs `malloc`, `snprintf`, multi-step allocation -> C in `builtin_qbe.c`

**Layer 2 (infer_builtin):** Currently only knows `len -> Int`. Must be expanded with return types for ALL builtins (both existing and new).

**Layer 3:** Antimony wrapper functions with explicit return type annotations. These are parsed by the builder and appear in the symbol table, so type inference for user code calling these wrappers already works (confirmed: `let n = int_to_str(42)` compiles correctly because `int_to_str` is in `lib/string.sb` with `: string` return type).

### Recommended New File Organization

```
src/generator/qbe.rs          # Add new QBE IL wrappers to RUNTIME_PREAMBLE
builtin/builtin_qbe.c         # Add C builtins needing malloc
lib/string.sb                  # Add str_char_at, str_substr wrappers
lib/io.sb                      # Keep as-is OR add file I/O wrappers here
src/parser/infer.rs            # Expand infer_builtin() with all return types
tests/qbe/                     # New test files per feature
```

### argc/argv Architecture

QBE's `main` function follows the C ABI. Currently generated as:
```
export function $main() { ... }
```

Must change to:
```
export function w $main(w %argc, l %argv) { ... }
```

**The challenge:** Antimony's `fn main()` has no parameters. The QBE generator must:
1. Detect when generating the `main` function
2. Emit the `(w %argc, l %argv)` parameters regardless of AST
3. Stash argc/argv into module-level globals at function entry
4. `_argc()` and `_argv(i)` read from those globals

**Implementation pattern (QBE IL globals):**
```
data $__argc = { w 0 }
data $__argv = { l 0 }

# At start of main:
storew %argc, $__argc
storel %argv, $__argv

# _argc() builtin:
export function w $_argc() {
@start
    %n =w loadw $__argc
    ret %n
}

# _argv(i) builtin -- returns pointer to i-th C string:
export function l $_argv(w %i) {
@start
    %base =l loadl $__argv
    %offset =l extsw %i
    %scaled =l mul %offset, 8
    %addr =l add %base, %scaled
    %ptr =l loadl %addr
    ret %ptr
}
```

### String Comparison Architecture

Currently, `==` and `!=` on all types emit `ceqw`/`cnew` (word comparison). For strings (which are pointers), this compares pointer identity, not string content.

**Fix approach:** In the QBE codegen's BinOp handler (~line 1138-1166 of `qbe.rs`), when the `Equal` or `NotEqual` operator is applied and both sides are string-typed (`qbe::Type::Long` with AST type `Str`), emit a call to `_strcmp` instead of a `ceqw` instruction.

The `is_string_expression()` method (line 1738) already exists for detecting string operands. The string comparison path should:
1. Call `$strcmp(l %lhs, l %rhs)` which returns `w` (0 for equal, nonzero otherwise)
2. For `==`: compare result with 0 using `ceqw`
3. For `!=`: compare result with 0 using `cnew`

### Method Return Type Inference Fix

**Root cause:** `infer_expression` in `infer.rs` (line 136) returns `None` for `FieldAccess` expressions. When the parser sees `let v = obj.method()`, it creates:
```
FieldAccess {
    expr: Variable("obj"),        // the receiver
    field: FunctionCall {          // the method call
        fn_name: "get_x",
        args: []
    }
}
```

`infer_expression` hits the `_ => None` catch-all. To fix:
1. Add a `HExpression::FieldAccess { expr, field }` arm to `infer_expression`
2. When `field` is a `FunctionCall`, resolve the receiver's struct type from `var_map`
3. Look up the struct definition in the module's struct list
4. Find the method by name and return its `ret_type`

**Complication:** The current `infer_expression` signature only has access to `table: &SymbolTable` (function name -> return type) and `var_map` (local variables). It does NOT have access to struct definitions. The SymbolTable is built from `HModule::get_symbol_table()` which only includes top-level functions, not struct methods.

**Solution options:**
- Option A (recommended): Extend `get_symbol_table()` to include struct methods as `StructName_methodName -> return type` entries, matching the QBE generator's naming convention. This is minimally invasive -- method calls already get mangled to `Foo_get_x` format.
- Option B: Pass struct definitions through the inference pipeline. More correct but more invasive refactor.

### Anti-Patterns to Avoid
- **Do not add C builtins for operations expressible in QBE IL.** Simple libc forwarding (strcmp, fopen, fclose, malloc) should be QBE IL preamble functions. Only use C when malloc + complex logic is needed.
- **Do not modify the parser broadly for the self.field fix.** The bug is already fixed -- just update the test to use direct assignment instead of the workaround.
- **Do not add return type inference to the QBE generator.** The `infer_fn_return_type` helper in `qbe.rs` is a workaround that should be replaced by fixing `infer.rs` properly, so all backends benefit.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| String comparison | Custom char-by-char comparison in QBE IL | libc `strcmp` via preamble wrapper | strcmp handles null terminators, locale, edge cases |
| String indexing | Pointer arithmetic in QBE IL | C helper in `builtin_qbe.c` | Bounds checking, null terminator awareness |
| Substring extraction | Manual malloc + copy in QBE IL | C helper with `malloc` + `memcpy` | Memory management too complex for QBE IL |
| File reading | Byte-by-byte read loop | libc `fgets` or `fread` via wrapper | Buffering, error handling, EOF detection |
| CLI arg parsing | Custom argc/argv decoding | Direct C ABI passthrough | QBE already implements C ABI in full |

## Common Pitfalls

### Pitfall 1: String Type Detection in BinOp Codegen
**What goes wrong:** Emitting `ceqw` for string equality instead of `strcmp` call
**Why it happens:** Both integers and string pointers are represented as QBE types (`Word` for int, `Long` for string pointers). Without checking the AST type, the generator cannot distinguish them.
**How to avoid:** Use the existing `is_string_expression()` method and/or the `fn_param_ast_types` maps to detect string operands before emitting comparison instructions.
**Warning signs:** String equality tests pass pointer comparison instead of content comparison.

### Pitfall 2: argc/argv Lifetime
**What goes wrong:** argc/argv pointers become invalid after main's stack frame is modified
**Why it happens:** If argc/argv are stored as stack locals, they share the stack with other main() locals
**How to avoid:** Store in module-level `data` globals (persistent for program lifetime), not stack allocs. QBE `data` definitions live in the data segment.

### Pitfall 3: QBE Word vs Long for Pointer Types
**What goes wrong:** Using `w` (32-bit word) for pointers on 64-bit systems
**Why it happens:** Antimony's `int` type maps to `w` (Word), but pointers (strings, malloc return) must be `l` (Long) on 64-bit
**How to avoid:** All pointer-returning builtins must use `l` return type. `malloc` returns `l`, not `w`. String operations return `l`.

### Pitfall 4: Missing fflush Before exit
**What goes wrong:** Output from printf/write is lost when program exits
**Why it happens:** `_exit()` / `_Exit()` does not flush stdio buffers
**How to avoid:** The existing `_exit` wrapper already calls `fflush(0)` before `_Exit`. Ensure all exit paths go through this wrapper.

### Pitfall 5: infer_builtin vs Symbol Table Coverage
**What goes wrong:** Adding builtins to `infer_builtin()` that are already covered by stdlib wrappers
**Why it happens:** Functions like `int_to_str` exist in `lib/string.sb` with return types. They appear in the symbol table. `infer_builtin` is only called as a fallback when the function is NOT in the symbol table.
**How to avoid:** Only underscore-prefixed builtins (`_strlen`, `_printf`, `_fopen`, etc.) need `infer_builtin` entries. Public Antimony wrappers are already covered by the symbol table.

## Code Examples

### Adding a QBE IL Preamble Wrapper (existing pattern)

```rust
// Source: src/generator/qbe.rs, RUNTIME_PREAMBLE (~line 35)
// Pattern: thin libc wrapper in QBE IL
const RUNTIME_PREAMBLE: &str = r#"
# _strcmp(a: l, b: l): w — compare two C strings
export function w $_strcmp(l %a, l %b) {
@start
    %r =w call $strcmp(l %a, l %b)
    ret %r
}
"#;
```

### Adding a C Builtin (existing pattern)

```c
// Source: builtin/builtin_qbe.c
// Pattern: operations needing malloc or complex logic
char *_str_char_at(char *s, long idx)
{
    // Return single character as a new 2-byte string
    char *buf = malloc(2);
    buf[0] = s[idx];
    buf[1] = '\0';
    return buf;
}

char *_str_substr(char *s, long start, long len)
{
    char *buf = malloc(len + 1);
    memcpy(buf, s + start, len);
    buf[len] = '\0';
    return buf;
}
```

### Expanding infer_builtin (fix pattern)

```rust
// Source: src/parser/infer.rs, infer_builtin (~line 165)
fn infer_builtin(name: &str) -> Option<Type> {
    match name {
        "len" | "_strlen" | "_parse_int" | "str_len" | "to_int" => Some(Type::Int),
        "_str_concat" | "_int_to_str" | "_read_line" | "_str_char_at" | "_str_substr" => {
            Some(Type::Str)
        }
        "_printf" | "_exit" => None, // void return
        "argc" | "_argc" => Some(Type::Int),
        "argv" | "_argv" => Some(Type::Str),
        "_malloc" => Some(Type::Int), // returns pointer-as-int
        "_fopen" => Some(Type::Int),  // returns FILE* as opaque int
        "_fclose" => Some(Type::Int), // returns status code
        _ => None,
    }
}
```

### Method Return Type Inference (fix pattern)

```rust
// Source: src/parser/infer.rs, add arm to infer_expression
HExpression::FieldAccess { expr, field } => {
    // Method call: obj.method() — resolve receiver type, look up method return type
    if let HExpression::FunctionCall { fn_name, .. } = field.as_ref() {
        // Get receiver's struct type from var_map
        if let Some(Type::Struct(struct_name)) = infer_expression(expr, table, var_map) {
            // Look up mangled method name in symbol table
            let mangled = format!("{}_{}", struct_name, fn_name);
            return table.get(&mangled).and_then(|t| t.clone());
        }
    }
    // Field access (not method call): would need struct field type lookup
    None
}
```

### argc/argv Global Stash Pattern

```
# QBE IL — emitted as part of RUNTIME_PREAMBLE or generated inline
data $__argc = { w 0 }
data $__argv = { l 0 }

# Modified main function signature (generated by QBE codegen):
export function w $main(w %argc, l %argv) {
@start
    storew %argc, $__argc
    storel %argv, $__argv
    # ... rest of user's main body ...
}
```

## Bug Status Update (from Phase 1 Gap Inventory)

| Bug | Status as of b6e32be | Action Required |
|-----|---------------------|-----------------|
| Bool codegen (wrong exit code) | FIXED -- test_booleans.sb passes | Update gap inventory, no code change |
| And/Or operators (runtime wrong) | FIXED -- test_booleans.sb passes | Update gap inventory, no code change |
| Str type inference (`int_to_str`) | FIXED -- stdlib wrappers have return types | Expand `infer_builtin()` for underscore-prefixed builtins |
| Method return type inference | STILL BROKEN -- `let v = obj.method()` fails | Fix `infer_expression` for FieldAccess; fix `get_symbol_table` |
| self.field = expr parser bug | FIXED -- direct assignment works | Update test_methods.sb to use direct assignment |

**Net impact:** Only 1 of 5 bugs needs actual fixing (method return type inference). The other 4 are documentation/test updates.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `infer_builtin` only knows `len` | Needs expansion to all builtins | Phase 2 | Prevents inference failures for raw builtins |
| QBE generator has own `infer_fn_return_type` | Should be in parser `infer.rs` | Phase 2 | All backends benefit from inference |
| `test_methods.sb` uses `self.value += 1` workaround | Direct `self.value = expr` works | b6e32be | Update test to validate direct assignment |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test (`#[test]`) + self-checking .sb programs |
| Config file | `Cargo.toml` (test profile) |
| Quick run command | `cargo test test_qbe_execution_tests -- --nocapture` |
| Full suite command | `cargo test -- --nocapture` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| RUNTIME-01 | String char access by index | integration | `cargo test test_qbe_execution_tests -- --nocapture` | New: `tests/qbe/test_string_ops.sb` |
| RUNTIME-02 | String equality comparison | integration | `cargo test test_qbe_execution_tests -- --nocapture` | New: `tests/qbe/test_string_compare.sb` |
| RUNTIME-03 | Substring extraction | integration | `cargo test test_qbe_execution_tests -- --nocapture` | New: `tests/qbe/test_string_ops.sb` (combined with RUNTIME-01) |
| RUNTIME-04 | File I/O (open/read/write/close) | integration | `cargo test test_qbe_execution_tests -- --nocapture` | New: `tests/qbe/test_file_io.sb` |
| RUNTIME-05 | CLI args (argc/argv) | integration | Manual: build + run with args | New: `tests/qbe/test_cli_args.sb` (needs special runner) |
| RUNTIME-06 | Heap allocation (malloc) | integration | `cargo test test_qbe_execution_tests -- --nocapture` | New: `tests/qbe/test_malloc.sb` |

**Note on RUNTIME-05:** The standard test harness runs binaries without arguments. Testing argc/argv requires a custom test that passes arguments, or extending `compile_and_run_qbe_checked` to accept arguments.

### Sampling Rate
- **Per task commit:** `cargo test test_qbe_execution_tests -- --nocapture`
- **Per wave merge:** `cargo test -- --nocapture`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/qbe/test_string_ops.sb` -- covers RUNTIME-01 and RUNTIME-03
- [ ] `tests/qbe/test_string_compare.sb` -- covers RUNTIME-02
- [ ] `tests/qbe/test_file_io.sb` -- covers RUNTIME-04
- [ ] `tests/qbe/test_cli_args.sb` -- covers RUNTIME-05 (needs custom runner or wrapper script)
- [ ] `tests/qbe/test_malloc.sb` -- covers RUNTIME-06
- [ ] `tests/qbe/test_method_inference.sb` -- covers method return type inference fix

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| QBE | All QBE compilation | Yes | installed at /opt/homebrew/bin/qbe | -- |
| C compiler (cc) | Linking QBE output | Yes | Apple clang 21.0.0 | -- |
| Rust/Cargo | Compiler build | Yes | 1.93.0 | -- |
| libc (strcmp, fopen, etc.) | Runtime builtins | Yes | System libc | -- |

**Missing dependencies:** None.

## Open Questions

1. **File I/O: fgets vs fread for text file reading**
   - What we know: `fgets` reads line-by-line (includes newline), `fread` reads N bytes (binary-safe). The bootstrap compiler needs to read source files.
   - Recommendation: Use `fgets` for line-oriented reading AND provide a `_fread_all` C helper that reads entire file contents into a malloc'd buffer. The bootstrap lexer will need whole-file reading.

2. **malloc return type in Antimony's type system**
   - What we know: `malloc` returns a `void*` (pointer). Antimony has no pointer type. The QBE generator uses `l` (Long/64-bit) for all pointers.
   - Recommendation: Return `int` from the Antimony-level `malloc` builtin, treating it as an opaque handle. This is the same approach used for FILE* handles from fopen. Phase 3 stdlib will wrap malloc in typed dynamic arrays.

3. **String comparison for `==` operator vs explicit function**
   - What we know: Currently `==` on strings compares pointer identity. Correct string equality needs `strcmp`.
   - Recommendation: Modify QBE codegen to emit `strcmp` call when `==`/`!=` operands are string-typed. This is the most ergonomic approach and matches user expectations. Also provide explicit `str_eq(a, b)` wrapper for clarity.

## Project Constraints (from CLAUDE.md)

- **Tech Stack:** QBE as the primary backend -- all systems-level work must target QBE
- **Bootstrap:** The bootstrapped compiler must be a full rewrite compiled via QBE
- **GSD Workflow:** Must use GSD commands for file changes
- **Code Style:** Rust standard formatting, 4-space indentation, clippy enforced with `-D warnings`
- **Error Handling:** `Result<T, String>` pattern, `GeneratorResult<T>` type alias
- **Naming:** Snake case for functions/variables, PascalCase for types/enums
- **Testing:** Test functions prefixed with `test_`, self-checking .sb programs for QBE

## Sources

### Primary (HIGH confidence)
- Direct codebase analysis: `src/generator/qbe.rs`, `src/parser/infer.rs`, `builtin/builtin_qbe.c`, `lib/*.sb`
- Live testing: compiled and ran test programs to verify bug status (booleans PASS, strings PASS, self.field PASS, method inference STILL FAILS)
- QBE IL reference: https://c9x.me/compile/doc/il.html -- function signatures, C ABI, call semantics, data definitions

### Secondary (MEDIUM confidence)
- [QBE Compiler Backend](https://c9x.me/compile/) -- general architecture and ABI compliance
- [acwj QBE tutorial](https://github.com/DoctorWkt/acwj/blob/master/63_QBE/Readme.md) -- argc/argv pattern in QBE main

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all changes use existing Rust/QBE/C toolchain, no new dependencies
- Architecture: HIGH -- patterns verified against working codebase; 3-layer builtin registration is well-established
- Pitfalls: HIGH -- derived from direct testing and code analysis of actual bugs
- Bug status: HIGH -- verified by compiling and running test programs locally

**Research date:** 2026-04-01
**Valid until:** 2026-05-01 (stable -- changes are in project's own codebase, not external dependencies)
