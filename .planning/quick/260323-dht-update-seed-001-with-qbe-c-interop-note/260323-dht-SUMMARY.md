---
phase: quick
plan: 260323-dht
subsystem: planning/seeds
tags: [ffi, qbe, documentation, seed]
key-files:
  modified:
    - .planning/seeds/SEED-001-ffi-c-interop.md
decisions:
  - "QBE IL handles external symbol linkage natively — FFI work is frontend-only"
metrics:
  duration: ~5min
  completed: 2026-03-23
  tasks: 1
  files: 1
---

# Quick Task 260323-dht: Update SEED-001 with QBE C interop note

**One-liner:** Expanded SEED-001 Notes to document QBE's native external symbol linkage and enumerate the frontend-only work items (lexer, parser, AST, transformer, generator body-skip).

## What Was Done

Updated `.planning/seeds/SEED-001-ffi-c-interop.md` to replace a single vague sentence in the Notes section with two clearly separated points:

1. **QBE IL handles external symbol linkage natively** — `call $symbol(...)` works for any C symbol without ABI shims or linker scripts. The generator change is minimal.
2. **Antimony's work is purely in the frontend** — itemized 6 concrete steps (lexer keyword, parser syntax, AST variant, transformer pass-through, generator body-skip, builder skip for extern imports).

Also updated the Scope Estimate paragraph to acknowledge the QBE simplification and note that a single focused phase may be sufficient rather than "a phase or two."

## Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Expand SEED-001 Notes with QBE C interop insight | abd4521 | .planning/seeds/SEED-001-ffi-c-interop.md |

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- .planning/seeds/SEED-001-ffi-c-interop.md: FOUND, contains "QBE IL handles external symbol linkage natively"
- Commit abd4521: FOUND
