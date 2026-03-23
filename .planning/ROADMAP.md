# Roadmap: Antimony Bootstrap Milestone

## Overview

This roadmap takes the Antimony compiler from its current state (Rust compiler with a partially-capable QBE backend) to a fully self-hosted compiler written in Antimony and compiled via QBE. The path is: stabilize what exists, build the missing runtime and stdlib primitives, then write the compiler in Antimony in two phases (frontend, then backend), and verify the bootstrap round-trip. Every phase before the compiler rewrite exists to ensure that rewrite can succeed without hitting language gaps mid-stream.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: QBE Stabilization and Audit** - Establish a trusted QBE backend with execution tests and a complete gap inventory
- [ ] **Phase 2: Runtime Primitives** - String operations, file I/O, CLI args, and heap allocation working in QBE
- [ ] **Phase 3: Standard Library** - Dynamic arrays, associative arrays, and string builder available for QBE target
- [ ] **Phase 4: Self-Hosted Compiler Frontend** - Lexer and parser implemented in Antimony, producing verified token streams and ASTs
- [ ] **Phase 5: Self-Hosted Compiler Backend and Builder** - Transform, QBE generator, builder, and main entry point complete; Stage 1 binary works
- [ ] **Phase 6: Bootstrap Verification** - Stage 2 round-trip verified; bootstrap milestone complete

## Phase Details

### Phase 1: QBE Stabilization and Audit
**Goal**: The QBE backend is trustworthy -- execution tests catch regressions and all language feature gaps are documented
**Depends on**: Nothing (first phase)
**Requirements**: STAB-01, STAB-02, STAB-03
**Success Criteria** (what must be TRUE):
  1. Programs compiled via QBE are linked and executed in CI, not just IL text-checked
  2. The unsafe transmute in QBE codegen is replaced with correct code or guarded with tests that prove it produces valid output for nested structs
  3. Every language feature has been tested against QBE codegen and gaps are catalogued in a document with severity and priority
**Plans**: 3 plans

Plans:
- [ ] 01-01-PLAN.md -- Build QBE execution test harness and core test programs (types, arithmetic, control flow, functions)
- [ ] 01-02-PLAN.md -- Complete language feature sweep (arrays, structs, methods, loops) and compile QBE-GAPS.md gap inventory
- [ ] 01-03-PLAN.md -- Fix upstream qbe crate lifetime issue and remove unsafe transmutes from generator

### Phase 2: Runtime Primitives
**Goal**: Antimony programs compiled via QBE can manipulate strings, read/write files, accept CLI arguments, and allocate heap memory
**Depends on**: Phase 1
**Requirements**: RUNTIME-01, RUNTIME-02, RUNTIME-03, RUNTIME-04, RUNTIME-05, RUNTIME-06
**Success Criteria** (what must be TRUE):
  1. An Antimony program can index into a string by position, extract substrings, and compare strings for equality -- all compiled via QBE and producing correct results
  2. An Antimony program can open a file, read its contents, write output to another file, and close both -- compiled via QBE
  3. An Antimony program can access command-line arguments (argc/argv equivalent)
  4. An Antimony program can allocate heap memory, use it for a dynamic data structure, and the program runs correctly (leak-everything is acceptable)
**Plans**: TBD

### Phase 3: Standard Library
**Goal**: The data structures needed to write a compiler (growable arrays, key-value lookup, efficient string building) are available as Antimony stdlib for QBE
**Depends on**: Phase 2
**Requirements**: STDLIB-01, STDLIB-02, STDLIB-03
**Success Criteria** (what must be TRUE):
  1. An Antimony program can create a growable array, push elements, access by index, and iterate -- compiled via QBE
  2. An Antimony program can store and retrieve key-value pairs using an associative array (linear search acceptable)
  3. An Antimony program can build a string incrementally (appending pieces) without quadratic copying
  4. A language feature freeze is in effect -- the tagged-struct convention for enums is documented and tested with Token/Statement/Expression definitions, or first-class enums are implemented
**Plans**: TBD

### Phase 4: Self-Hosted Compiler Frontend
**Goal**: The Antimony lexer and parser exist as Antimony source files, compiled via QBE, and produce output identical to the Rust compiler's frontend
**Depends on**: Phase 3
**Requirements**: BOOT-01, BOOT-02
**Success Criteria** (what must be TRUE):
  1. The Antimony lexer (sb/lexer.sb) tokenizes Antimony source files and produces token streams that match the Rust compiler's output on test programs
  2. The Antimony parser (sb/parser.sb) produces annotated ASTs from token streams, with type inference, matching the Rust compiler's output on test programs
  3. AST type definitions (Token, Statement, Expression, Type) exist as Antimony structs and can represent the full language
**Plans**: TBD

### Phase 5: Self-Hosted Compiler Backend and Builder
**Goal**: A complete Antimony compiler exists as Antimony source, and the Rust compiler can compile it into a working Stage 1 binary
**Depends on**: Phase 4
**Requirements**: BOOT-03, BOOT-04
**Success Criteria** (what must be TRUE):
  1. The Antimony transform (sb/transform.sb) lowers high-level AST to low-level AST, matching the Rust compiler's lowering
  2. The Antimony QBE generator (sb/generator_qbe.sb) emits SSA text that, when compiled via qbe and gcc, produces working binaries
  3. The builder (sb/builder.sb) handles file loading, import resolution, and module merging
  4. A Stage 1 binary (produced by the Rust compiler from the Antimony source) can compile at least one non-trivial Antimony program end-to-end
**Plans**: TBD

### Phase 6: Bootstrap Verification
**Goal**: The bootstrap is proven correct -- Stage 1 compiles itself into Stage 2 and the output is functionally equivalent
**Depends on**: Phase 5
**Requirements**: BOOT-05
**Success Criteria** (what must be TRUE):
  1. Stage 1 binary compiles the Antimony compiler source (sb/*.sb) and produces a Stage 2 binary
  2. Stage 2 binary passes the full test suite (all tests that Stage 1 passes)
  3. A Makefile with stage1, stage2, and verify targets exists and the verify target succeeds
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. QBE Stabilization and Audit | 0/3 | Planned    |  |
| 2. Runtime Primitives | 0/TBD | Not started | - |
| 3. Standard Library | 0/TBD | Not started | - |
| 4. Self-Hosted Compiler Frontend | 0/TBD | Not started | - |
| 5. Self-Hosted Compiler Backend and Builder | 0/TBD | Not started | - |
| 6. Bootstrap Verification | 0/TBD | Not started | - |

## Backlog

### Phase 999.1: long-term runtime performance tracking and benchmark suite (BACKLOG)

**Goal:** [Captured for future planning]
**Requirements:** TBD
**Plans:** 0/3 plans executed

Plans:
- [ ] TBD (promote with /gsd:review-backlog when ready)

### Phase 999.2: Package manager for Antimony (BACKLOG)

**Goal:** [Captured for future planning]
**Requirements:** TBD
**Plans:** 0 plans

Plans:
- [ ] TBD (promote with /gsd:review-backlog when ready)

### Phase 999.3: LSP; VSCode & neovim plugins (BACKLOG)

**Goal:** [Captured for future planning]
**Requirements:** TBD
**Plans:** 0 plans

Plans:
- [ ] TBD (promote with /gsd:review-backlog when ready)
