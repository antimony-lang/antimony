---
phase: 02-runtime-primitives
plan: 01
subsystem: compiler
tags: [qbe, type-inference, strcmp, string-comparison, codegen]

# Dependency graph
requires:
  - phase: 01-qbe-stabilization
    provides: QBE execution test harness and gap audit
provides:
  - Method return type inference without explicit annotations
  - Complete builtin return type coverage for type inference
  - Content-based string comparison via strcmp in QBE codegen
  - _strcmp QBE IL preamble wrapper
affects: [02-02, 02-03, all future QBE plans]

# Tech tracking
tech-stack:
  added: []
  patterns: [strcmp-based string comparison, mangled method names in symbol table]

key-files:
  created:
    - tests/qbe/test_method_inference.sb
    - tests/qbe/test_string_compare.sb
  modified:
    - src/parser/infer.rs
    - src/ast/hast.rs
    - src/generator/qbe.rs

key-decisions:
  - "Method names mangled as StructName_methodName in symbol table, matching QBE generator convention"
  - "String comparison uses _strcmp wrapper calling libc strcmp, returning word-width result"

patterns-established:
  - "Builtin inference: all underscore-prefixed builtins registered in infer_builtin with correct return types"
  - "String operator dispatch: is_string_expression check before BinOp codegen to route to strcmp path"

requirements-completed: [RUNTIME-02]

# Metrics
duration: 4min
completed: 2026-04-01
---

# Phase 02 Plan 01: Type Inference and String Comparison Summary

**Method return type inference via mangled symbol table lookups and strcmp-based string content comparison in QBE codegen**

## Performance

- **Duration:** 4 min
- **Started:** 2026-04-01T12:27:54Z
- **Completed:** 2026-04-01T12:31:27Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Method return types now inferred without explicit type annotations (e.g. `let v = obj.method()` compiles)
- All underscore-prefixed builtins (_strlen, _str_concat, _int_to_str, _read_line, _parse_int, _printf, _exit) have correct return types in infer_builtin
- String == and != operators compare content via libc strcmp instead of pointer identity
- 17/17 QBE execution tests pass (up from 15 in previous phase), full 205-test suite green

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix type inference -- method return types and builtin return types** - `c1bf8a0` (feat)
2. **Task 2: Implement string comparison via strcmp in QBE codegen** - `406417d` (feat)

## Files Created/Modified
- `src/ast/hast.rs` - get_symbol_table now includes struct methods with mangled names
- `src/parser/infer.rs` - FieldAccess inference arm, expanded infer_builtin, 7 new unit tests
- `src/generator/qbe.rs` - _strcmp preamble wrapper, strcmp-based == and != for strings
- `tests/qbe/test_method_inference.sb` - Self-checking test for method return type inference
- `tests/qbe/test_string_compare.sb` - Self-checking test for string ==, !=, variable, and concat comparisons

## Decisions Made
- Method names mangled as StructName_methodName in the symbol table, matching the existing QBE generator convention for method dispatch
- String comparison uses a thin _strcmp QBE IL wrapper calling libc strcmp, with the result compared to 0 for == (Eq) and != (Ne)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Type inference foundation is solid for Plans 02 and 03 which add new builtins
- String comparison enables correct string-based control flow needed throughout Phase 2
- All 17 QBE tests pass with zero regressions

---
*Phase: 02-runtime-primitives*
*Completed: 2026-04-01*
