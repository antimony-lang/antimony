---
phase: 1
slug: qbe-stabilization-and-audit
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test framework (cargo test) |
| **Config file** | None — uses `#[test]` attributes inline |
| **Quick run command** | `cargo test test_qbe -- --nocapture` |
| **Full suite command** | `cargo test -- --nocapture` |
| **Estimated runtime** | ~60 seconds (includes QBE compile + link) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test test_qbe -- --nocapture`
- **After every plan wave:** Run `cargo test -- --nocapture`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 1-01-01 | 01 | 1 | STAB-02 | compile | `cargo build 2>&1 && ! grep -c 'transmute' src/generator/qbe.rs` | ❌ W0 | ⬜ pending |
| 1-02-01 | 02 | 1 | STAB-01 | integration | `cargo test test_qbe_execution_tests -- --nocapture` | ❌ W0 | ⬜ pending |
| 1-03-01 | 03 | 2 | STAB-03 | manual + integration | `cargo test test_qbe -- --nocapture` + verify QBE-GAPS.md | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/qbe/` directory — does not exist yet
- [ ] Test harness function for `tests/qbe/` discovery — extend `src/tests/test_examples.rs` with `compile_and_run_qbe_checked()` (checks exit code 0 AND scans stdout for "FAIL")
- [ ] At least one `.sb` self-checking test to validate the harness works before writing all 18 feature tests

*Wave 0 is part of Plan 02 (test harness plan).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| QBE-GAPS.md covers all LAST enum variants | STAB-03 | Gap inventory completeness requires human review of feature table | Open `.planning/phases/01-qbe-stabilization-and-audit/QBE-GAPS.md`; verify rows exist for all 9 Statement, 12 Expression, 18 BinOp, and 6 Type variants from `src/ast/last.rs` and `src/ast/types.rs` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
