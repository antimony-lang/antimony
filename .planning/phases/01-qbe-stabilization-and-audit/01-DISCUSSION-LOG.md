# Phase 1: QBE Stabilization and Audit - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 01-qbe-stabilization-and-audit
**Areas discussed:** Execution test strategy, Transmute resolution, Gap inventory format, Test program coverage

---

## Execution Test Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Expand integration test suite | Add more .sb programs that compile and run through QBE, check stdout/exit code. Keep unit tests as IL snapshots. | ✓ |
| Convert unit tests to execution tests | Each unit test should compile its generated IL through QBE+gcc and run the binary | |
| Both | Keep IL snapshot tests AND add execution tests for the same cases | |
| You decide | Claude's discretion | |

**User's choice:** Expand integration test suite
**Notes:** Matches STAB-01 directly and builds on existing test_examples_qbe pattern.

### Follow-up: Validation method

| Option | Description | Selected |
|--------|-------------|----------|
| Expected stdout | Each test program prints output, test compares against expected string | |
| Exit code | Programs return 0 for pass, non-zero for fail | |
| Both | Check stdout AND exit code | ✓ |

**User's choice:** Both — most thorough

### Follow-up: Test location

| Option | Description | Selected |
|--------|-------------|----------|
| tests/qbe/ directory | Dedicated directory for QBE execution tests, separate from examples | ✓ |
| Expand examples/ | Add more example programs that double as QBE tests | |

**User's choice:** tests/qbe/ directory — keeps examples clean for documentation

---

## Transmute Resolution

| Option | Description | Selected |
|--------|-------------|----------|
| Fix upstream in qbe crate | Submit a PR to the qbe crate to relax lifetime constraints or add owned type variant | ✓ |
| Refactor to avoid transmute | Restructure the generator to satisfy the borrow checker without transmute | |
| Add proving tests | Keep the transmutes but add tests that exercise nested structs/arrays heavily | |

**User's choice:** Fix upstream in qbe crate
**Notes:** User revealed they own the qbe crate — this is a direct upstream fix, not a third-party PR. No fallback needed; user will merge the fix once it's ready.

---

## Gap Inventory Format

| Option | Description | Selected |
|--------|-------------|----------|
| Feature checklist with severity | Table listing every language feature, QBE status (pass/fail/partial), severity | ✓ |
| Categorized matrix | Group by domain with columns for feature, QBE status, test coverage, blocking phase, notes | |
| Narrative per feature | Short write-up for each gap explaining what's missing, why it matters, rough effort | |

**User's choice:** Feature checklist with severity — simple and scannable

### Follow-up: Location

| Option | Description | Selected |
|--------|-------------|----------|
| .planning/phases/01-.../QBE-GAPS.md | Phase artifact, colocated with other phase docs | ✓ |
| docs/qbe-gaps.md | In the project docs, visible to anyone browsing the repo | |

**User's choice:** Phase artifact

### Follow-up: Phase cross-references

| Option | Description | Selected |
|--------|-------------|----------|
| Yes | Cross-reference each gap to the roadmap phase that resolves it | ✓ |
| No | Just list gaps with severity | |

**User's choice:** Yes — makes it a planning tool, not just a list

---

## Test Program Coverage

| Option | Description | Selected |
|--------|-------------|----------|
| Systematic language feature sweep | Write a test program for every language feature | ✓ |
| Compiler-focused patterns only | Test only patterns the self-hosted compiler will use | |
| Both in layers | Systematic sweep first, then compiler-focused stress tests | |

**User's choice:** Systematic language feature sweep — need the full picture for the gap inventory

### Follow-up: File organization

| Option | Description | Selected |
|--------|-------------|----------|
| One file per feature | e.g., test_structs.sb, test_while_loops.sb | ✓ |
| Grouped by category | e.g., test_control_flow.sb covers if/else/while/for/break/continue | |

**User's choice:** One file per feature — easy to isolate failures

### Follow-up: Pass/fail signaling

| Option | Description | Selected |
|--------|-------------|----------|
| Self-checking | Each .sb does its own assertions, prints PASS/FAIL, exits 0 or 1 | ✓ |
| Expected output comparison | Each .sb has a companion .expected file | |

**User's choice:** Self-checking — test logic lives in the .sb file, harness stays simple

---

## Claude's Discretion

- Order of language features to test
- Internal structure of the test harness
- How to enumerate language features systematically

## Deferred Ideas

None — discussion stayed within phase scope
