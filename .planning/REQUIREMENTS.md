# Requirements: Antimony — Bootstrap Milestone

**Defined:** 2026-03-23
**Core Value:** The QBE backend must become capable enough that real systems programs — including the compiler itself — can be written in Antimony and compiled correctly.

## v1 Requirements

### QBE Stabilization

- [ ] **STAB-01**: End-to-end execution tests exist — programs are compiled via QBE, linked, and executed (not just IL text-checked)
- [ ] **STAB-02**: Unsafe transmute UB in QBE codegen is resolved
- [ ] **STAB-03**: Formal gap inventory completed — every language feature is tested for correct QBE codegen and gaps are documented

### Runtime Primitives

- [ ] **RUNTIME-01**: String character access works in QBE (index into string by position)
- [ ] **RUNTIME-02**: String comparison works in QBE (`==` on strings calls `strcmp`-equivalent)
- [ ] **RUNTIME-03**: Substring extraction works in QBE
- [ ] **RUNTIME-04**: File I/O primitives available in Antimony (open, read, write, close)
- [ ] **RUNTIME-05**: CLI arguments accessible from Antimony programs (argc/argv)
- [ ] **RUNTIME-06**: Heap allocation strategy decided and implemented (design deferred — resolved when first compiler rewrite use case demands it)

### Standard Library

- [ ] **STDLIB-01**: Dynamic arrays (growable) available in Antimony stdlib for QBE target
- [ ] **STDLIB-02**: Associative arrays (key-value, linear search acceptable) available for QBE target
- [ ] **STDLIB-03**: String builder / efficient string concatenation available for QBE target

### Self-Hosted Compiler

- [ ] **BOOT-01**: Lexer implemented in Antimony — produces token stream from source text
- [ ] **BOOT-02**: Parser and type inference implemented in Antimony — produces annotated AST from token stream
- [ ] **BOOT-03**: AST transform and QBE code generator implemented in Antimony — lowers AST and emits SSA text
- [ ] **BOOT-04**: Builder and main entry point implemented in Antimony — handles file loading, import resolution, CLI; produces working compiler binary (Stage 1)
- [ ] **BOOT-05**: Stage 2 round-trip verified — Stage 1 compiler compiles itself and the output matches Stage 1

## v2 Requirements

### Doom Milestone

- Doom scope TBD — deferred until bootstrap milestone reveals language capabilities

### Language Features (Post-Bootstrap)

- First-class enum / sum types — deferred; bootstrap will use struct-with-integer-tag pattern
- Generics — deferred; bootstrap will use untyped workarounds
- Garbage collection — deferred; bootstrap will use leak-everything approach for batch programs

## Out of Scope

| Feature | Reason |
|---------|--------|
| JS backend improvements | JS backend served its purpose; QBE is the focus |
| LLVM and x86 backends | Not the current focus; effort is on QBE maturity |
| Language syntax redesign | This milestone is about backend capability, not language evolution |
| Multi-backend in self-hosted compiler | The self-hosted compiler targets QBE only — no need to replicate all 5 backends |
| Real-time memory management (GC/RAII) | Adds massive complexity; compilers are batch programs that can leak |
| Generics before bootstrap | Use untyped (`any`) workarounds during bootstrap to avoid language redesign |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| STAB-01 | TBD | Pending |
| STAB-02 | TBD | Pending |
| STAB-03 | TBD | Pending |
| RUNTIME-01 | TBD | Pending |
| RUNTIME-02 | TBD | Pending |
| RUNTIME-03 | TBD | Pending |
| RUNTIME-04 | TBD | Pending |
| RUNTIME-05 | TBD | Pending |
| RUNTIME-06 | TBD | Pending |
| STDLIB-01 | TBD | Pending |
| STDLIB-02 | TBD | Pending |
| STDLIB-03 | TBD | Pending |
| BOOT-01 | TBD | Pending |
| BOOT-02 | TBD | Pending |
| BOOT-03 | TBD | Pending |
| BOOT-04 | TBD | Pending |
| BOOT-05 | TBD | Pending |
