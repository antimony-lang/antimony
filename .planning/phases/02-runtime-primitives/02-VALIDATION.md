---
phase: 02
slug: runtime-primitives
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-01
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust cargo test (integration) |
| **Config file** | Cargo.toml (existing) |
| **Quick run command** | `cargo test test_qbe` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test test_qbe`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 02-01-01 | 01 | 1 | RUNTIME-02 | integration | `cargo test test_qbe_execution_tests -- --nocapture` | ⬜ pending |
| 02-01-02 | 01 | 1 | RUNTIME-01 | integration | `cargo test test_qbe_execution_tests -- --nocapture` | ⬜ pending |
| 02-01-03 | 01 | 1 | RUNTIME-03 | integration | `cargo test test_qbe_execution_tests -- --nocapture` | ⬜ pending |
| 02-02-01 | 02 | 1 | RUNTIME-04 | integration | `cargo test test_qbe_execution_tests -- --nocapture` | ⬜ pending |
| 02-02-02 | 02 | 1 | RUNTIME-05 | integration | `cargo test test_qbe_execution_tests -- --nocapture` | ⬜ pending |
| 02-02-03 | 02 | 1 | RUNTIME-06 | integration | `cargo test test_qbe_execution_tests -- --nocapture` | ⬜ pending |
| 02-03-01 | 03 | 2 | RUNTIME-01..06 | unit+integration | `cargo test` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. The `tests/qbe/` directory and `compile_and_run_qbe_checked()` harness from Phase 1 are reused as-is. New .sb test files are added per feature (one per builtin/fix) following the established self-checking pattern.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| File I/O round-trip | RUNTIME-04 | Requires a real file on disk | Run `tests/qbe/test_file_io.sb`, verify output file matches input |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
