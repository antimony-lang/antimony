# QBE Backend Gap Inventory

**Generated:** 2026-03-31
**Method:** Systematic test sweep of all LAST enum variants via self-checking .sb programs
**Test directory:** tests/qbe/
**Total features tested:** 43
**Pass / Fail / Partial:** 37 / 5 / 1

## Summary

The QBE backend handles core imperative constructs well: integer arithmetic, comparisons, variables, control flow (if/else, while, for-in, match), functions (including recursion), arrays, structs with nested field access, and compound assignment all pass. The critical gaps are: (1) boolean expressions produce incorrect exit codes, (2) type inference fails for method return types and builtin function return types requiring explicit type annotations as workarounds, and (3) direct assignment to self fields in methods fails at parse time. String operations are blocked by the type inference gap. Overall confidence for bootstrap path: the core codegen is solid, but type inference and boolean handling must be fixed before self-hosting work begins.

## Feature Matrix

### Types

| Type | QBE Status | Severity | Resolving Phase | Test File | Notes |
|------|------------|----------|-----------------|-----------|-------|
| Int | PASS | - | - | test_int_arithmetic.sb | Fully working: literals, arithmetic, comparison |
| Str | FAIL | Blocks bootstrap | Phase 2: Runtime Primitives | test_strings.sb | String literals work in println(); type inference fails for `let n = int_to_str(42)` -- needs explicit type annotation |
| Bool | FAIL | Blocks bootstrap | Phase 2: Runtime Primitives | test_booleans.sb | Bool literals compile but binary exits non-zero; likely codegen issue with boolean-to-exit-code path |
| Any | PARTIAL | Needed later | Phase 3: Standard Library | - | No dedicated test; used implicitly in some contexts |
| Array(T, size) | PASS | - | - | test_arrays.sb | Array literal, index access, assignment, len() all work |
| Struct(name) | PASS | - | - | test_structs.sb | Init, field access, nested structs all work |

### Statements

| Statement | QBE Status | Severity | Resolving Phase | Test File | Notes |
|-----------|------------|----------|-----------------|-----------|-------|
| Block | PASS | - | - | test_functions.sb | Block scoping works correctly |
| Declare | PASS | - | - | test_variables.sb | `let` and `let x: type` both work |
| Assign | PASS | - | - | test_variables.sb | Variable reassignment works |
| Return | PASS | - | - | test_functions.sb | Return with value and void return both work |
| If | PASS | - | - | test_if_else.sb | if, if/else, if/else-if/else, nested all work |
| While | PASS | - | - | test_while_loops.sb | Basic while, nested while both work |
| For | PASS | - | - | test_for_loops.sb | for-in over arrays works (lowered to while) |
| Break | PASS | - | - | test_while_loops.sb | Break exits inner loop correctly |
| Continue | PASS | - | - | test_while_loops.sb | Continue skips iteration correctly |
| Exp | PASS | - | - | test_functions.sb | Expression statements (function calls) work |

### Expressions

| Expression | QBE Status | Severity | Resolving Phase | Test File | Notes |
|------------|------------|----------|-----------------|-----------|-------|
| Int(usize) | PASS | - | - | test_int_arithmetic.sb | Integer literals work correctly |
| Str(String) | PASS | - | - | test_strings.sb | String literals work in println(); gap is in type inference for string-returning functions |
| Bool(bool) | FAIL | Blocks bootstrap | Phase 2: Runtime Primitives | test_booleans.sb | Bool literals compile but produce wrong exit code |
| Selff | PARTIAL | Blocks bootstrap | Phase 2: Runtime Primitives | test_methods.sb | `self.field` reads work; `self.field = expr` direct assignment fails at parse time; `self.field += expr` works |
| Array { capacity, elements } | PASS | - | - | test_arrays.sb | Array literals with elements work |
| FunctionCall { fn_name, args } | PASS | - | - | test_functions.sb | Direct calls, recursive calls work |
| Variable(String) | PASS | - | - | test_variables.sb | Variable references work |
| ArrayAccess { name, index } | PASS | - | - | test_arrays.sb | Index access and assignment work |
| BinOp { lhs, op, rhs } | PASS | - | - | test_int_arithmetic.sb | See BinOps detail table below |
| StructInitialization | PASS | - | - | test_structs.sb | `new Struct { field: value }` works |
| FieldAccess | PASS | - | - | test_structs.sb | `obj.field` and `obj.nested.field` work |

### BinOps (detail)

| BinOp | Symbol | QBE Status | Test File | Notes |
|-------|--------|------------|-----------|-------|
| Addition | + | PASS | test_int_arithmetic.sb | Works for integers |
| Subtraction | - | PASS | test_int_arithmetic.sb | Works for integers |
| Multiplication | * | PASS | test_int_arithmetic.sb | Works for integers |
| Division | / | PASS | test_int_arithmetic.sb | Works for integers |
| Modulus | % | PASS | test_int_arithmetic.sb | Works for integers |
| LessThan | < | PASS | test_int_comparison.sb | Works for integers |
| LessThanOrEqual | <= | PASS | test_int_comparison.sb | Works for integers |
| GreaterThan | > | PASS | test_int_comparison.sb | Works for integers |
| GreaterThanOrEqual | >= | PASS | test_int_comparison.sb | Works for integers |
| Equal | == | PASS | test_int_comparison.sb | Works for integers |
| NotEqual | != | PASS | test_int_comparison.sb | Works for integers |
| And | && | FAIL | Blocks bootstrap | Phase 2: Runtime Primitives | test_booleans.sb | Part of boolean gap -- compiles but runtime behavior wrong |
| Or | \|\| | FAIL | Blocks bootstrap | Phase 2: Runtime Primitives | test_booleans.sb | Part of boolean gap -- compiles but runtime behavior wrong |
| AddAssign | += | PASS | test_compound_assign.sb | Works for integers and struct fields |
| SubtractAssign | -= | PASS | test_compound_assign.sb | Works for integers |
| MultiplyAssign | *= | PASS | test_compound_assign.sb | Works for integers |
| DivideAssign | /= | PASS | test_compound_assign.sb | Works for integers |

### High-Level Constructs (lowered before QBE)

| Construct | Lowered To | QBE Status | Test File | Notes |
|-----------|-----------|------------|-----------|-------|
| Match | If-else chain | PASS | test_match.sb | Integer match with cases and else branch works |
| For-in | While loop | PASS | test_for_loops.sb | for-in over arrays works correctly |

## Gaps Blocking Bootstrap (Phase 4-5)

| Feature | Gap Description | Resolving Phase |
|---------|-----------------|-----------------|
| Bool type | Boolean expressions compile but binary exits with wrong code; likely QBE codegen issue with boolean values | Phase 2: Runtime Primitives |
| And (&&) / Or (\|\|) | Logical operators are part of the boolean gap -- runtime behavior incorrect | Phase 2: Runtime Primitives |
| Str type inference | `let n = int_to_str(42)` fails with "Missing type for variable" -- type inference does not propagate return types from builtin/stdlib functions | Phase 2: Runtime Primitives |
| Method return type inference | `let v = obj.method()` fails with "Missing type for variable" -- type inference does not propagate method return types; workaround: explicit `let v: int = obj.method()` | Phase 2: Runtime Primitives |
| Self field direct assignment | `self.field = expr` fails at parse time; only `self.field += expr` compound assignment works | Phase 2: Runtime Primitives |

## Phase Cross-Reference

| Phase | Gaps to Resolve | Count |
|-------|----------------|-------|
| Phase 2: Runtime Primitives | Bool codegen, And/Or operators, Str type inference, method return type inference, self field assignment | 5 |
| Phase 3: Standard Library | Any type (dedicated support), dynamic arrays, associative arrays | 3 |
| Phase 4: Self-Hosted Frontend | No additional gaps (depends on Phase 2-3 fixes) | 0 |
| Phase 5: Self-Hosted Backend | No additional gaps (depends on Phase 2-3 fixes) | 0 |
