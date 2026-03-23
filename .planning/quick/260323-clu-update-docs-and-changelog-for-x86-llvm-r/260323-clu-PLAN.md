---
phase: quick
plan: 260323-clu
type: execute
wave: 1
depends_on: []
files_modified:
  - docs/developers/backends.md
  - CHANGELOG.md
autonomous: true
must_haves:
  truths:
    - "docs/developers/backends.md no longer mentions x86, LLVM, or ARM as planned/available backends"
    - "docs/developers/backends.md accurately lists only JS, C, and QBE as available backends"
    - "CHANGELOG.md has an entry under Unreleased recording the removal of x86 and LLVM backends"
    - "Historical changelog entries (v0.3.0 and earlier) remain unchanged"
  artifacts:
    - path: "docs/developers/backends.md"
      provides: "Updated backend documentation with only JS, C, QBE"
    - path: "CHANGELOG.md"
      provides: "Changelog entry for backend removal"
  key_links: []
---

<objective>
Update documentation and changelog to reflect that x86 and LLVM backends have been removed.

Purpose: The x86 and LLVM generator code, CLI options, dependencies, and tests were removed in quick task 260323-cfg. The docs and changelog still reference these removed backends and need to be brought in sync.
Output: Updated docs/developers/backends.md and CHANGELOG.md
</objective>

<execution_context>
@.planning/quick/260323-clu-update-docs-and-changelog-for-x86-llvm-r/260323-clu-PLAN.md
</execution_context>

<context>
@docs/developers/backends.md
@CHANGELOG.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Update backends documentation</name>
  <files>docs/developers/backends.md</files>
  <action>
Rewrite docs/developers/backends.md to reflect that only JS, C, and QBE backends exist:

1. Update the opening paragraph: remove "WASM, ARM and x86 are planned" and instead state that Antimony supports three backends: JavaScript, C, and QBE (with QBE being the primary systems-level target).

2. Update the "Available Backends" table: remove the LLVM row entirely. Keep JS, QBE, and C rows. Update stability notes if appropriate (QBE is the primary backend now, fix the "work in progess" typo to "work in progress").

3. Remove the entire LLVM feature-flag section (lines 22-26: "LLVM also requires..." and the cargo build command).

4. Keep the [QBE] link reference.

Do NOT change the CLI usage example (sb -t c build ...) as that is still valid.
  </action>
  <verify>
    <automated>grep -ciE 'llvm|x86|ARM' docs/developers/backends.md | grep -q '^0$' && echo "PASS: no removed backends mentioned" || echo "FAIL: still references removed backends"</automated>
  </verify>
  <done>backends.md lists only JS, C, and QBE with no mention of LLVM, x86, or ARM</done>
</task>

<task type="auto">
  <name>Task 2: Add changelog entry for backend removal</name>
  <files>CHANGELOG.md</files>
  <action>
Add a new entry under the existing "## Unreleased" section in CHANGELOG.md. Under the existing **Maintenance** subsection (create it if not present -- it already exists at line 44), add:

- Remove x86 and LLVM backends -- only C, JS, and QBE compilation targets remain

This goes in the **Maintenance** subsection since it is a codebase cleanup, not a feature or fix.

Do NOT modify any historical changelog entries (v0.3.0 "First attempt of LLVM backend", v0.0.1 shortlog mentioning x86 scaffolding, etc.) -- those are accurate historical records.
  </action>
  <verify>
    <automated>grep -q "Remove x86 and LLVM backends" CHANGELOG.md && echo "PASS: changelog entry exists" || echo "FAIL: missing changelog entry"</automated>
  </verify>
  <done>CHANGELOG.md Unreleased/Maintenance section includes the backend removal entry; all historical entries unchanged</done>
</task>

</tasks>

<verification>
- `grep -ciE 'llvm|x86|ARM' docs/developers/backends.md` returns 0
- `grep "Remove x86 and LLVM backends" CHANGELOG.md` finds the new entry
- Historical changelog entries for v0.3.0 and v0.0.1 are unchanged
</verification>

<success_criteria>
- docs/developers/backends.md accurately documents only JS, C, and QBE backends
- CHANGELOG.md records the removal under Unreleased > Maintenance
- No historical entries are modified
</success_criteria>

<output>
After completion, create `.planning/quick/260323-clu-update-docs-and-changelog-for-x86-llvm-r/260323-clu-SUMMARY.md`
</output>
