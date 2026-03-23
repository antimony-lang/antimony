---
phase: quick
plan: 260323-dht
type: execute
wave: 1
depends_on: []
files_modified:
  - .planning/seeds/SEED-001-ffi-c-interop.md
autonomous: true
requirements: []

must_haves:
  truths:
    - "SEED-001 Notes section explicitly states that QBE IL handles external symbol linkage natively"
    - "SEED-001 Notes section states Antimony only needs extern declaration syntax — the generator layer requires minimal change"
    - "The scope estimate comment in the seed reflects this simplification"
  artifacts:
    - path: ".planning/seeds/SEED-001-ffi-c-interop.md"
      provides: "Updated seed with QBE C interop note"
      contains: "QBE built-in C interop"
  key_links: []
---

<objective>
Update SEED-001 (FFI for Antimony) with an explicit note that QBE IL has built-in C interop — external symbol linkage is handled natively by QBE, so the bulk of the FFI work is in the Antimony parser/AST layer (`extern` declaration syntax) rather than the code generator.

Purpose: Preserve this scope-simplifying insight so that when SEED-001 is surfaced during a future milestone it accurately reflects the implementation complexity.
Output: Updated `.planning/seeds/SEED-001-ffi-c-interop.md` with an expanded Notes section.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/seeds/SEED-001-ffi-c-interop.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Expand SEED-001 Notes with QBE C interop insight</name>
  <files>.planning/seeds/SEED-001-ffi-c-interop.md</files>
  <action>
    Edit the Notes section of SEED-001 to make the QBE C interop point explicit and prominent. The current Notes section contains a single paragraph that buries the key insight. Replace it with two clearly separated points:

    1. **QBE IL handles external symbol linkage natively** — QBE's `call $symbol(...)` syntax works for any external C symbol without special configuration. No ABI shim or linker script is needed. This is the hard part of FFI and QBE already solves it. The generator change is therefore minimal: emit `call $fn_name(...)` for `extern` calls the same way it does for internal calls, without emitting a function body.

    2. **Antimony's work is purely in the frontend** — The remaining work is: (a) `extern` keyword in the lexer, (b) `extern fn name(args) -> ret` declaration syntax in the parser (natural anchor: `parse_declare()` in `src/parser/rules.rs:785`), (c) a new AST variant (e.g., `HStatement::ExternFn` or a flag on the existing function node) to represent a bodyless foreign declaration, (d) the transformer must pass extern declarations through to LAST without requiring a body, and (e) the QBE generator skips emitting a function body for extern-flagged functions. The builder must also skip trying to parse a source file for `extern` imports.

    Also update the Scope Estimate paragraph to note that because QBE handles the interop layer, the complexity is lower than a typical FFI implementation — the estimate of "a phase or two" may be conservative; a single focused phase is plausible.

    Preserve all existing content (frontmatter, Why This Matters, When to Surface, Breadcrumbs) unchanged. Only extend the Notes section and adjust the Scope Estimate prose.
  </action>
  <verify>
    Read the updated file and confirm:
    - Notes section contains explicit mention of QBE's native external symbol linkage
    - Notes section lists the frontend-only work items (lexer, parser, AST, transformer, generator body-skip)
    - Scope Estimate paragraph acknowledges QBE simplification
    - All other sections are unchanged
  </verify>
  <done>
    SEED-001 Notes section clearly states that QBE C interop is built-in, and the remaining FFI work is confined to Antimony's frontend (lexer/parser/AST/transformer) with only a trivial generator change.
  </done>
</task>

</tasks>

<verification>
Read `.planning/seeds/SEED-001-ffi-c-interop.md` after the task and confirm the Notes section contains both the QBE interop point and the itemized frontend work list.
</verification>

<success_criteria>
SEED-001 accurately reflects that QBE handles external C symbol linkage natively, scoping future FFI work to the Antimony frontend only.
</success_criteria>

<output>
No SUMMARY.md required for quick tasks. Commit the updated seed file.
</output>
