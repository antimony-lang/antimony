---
created: 2026-04-01T14:21:15.855Z
title: Document enum syntax and semantics in Antimony docs
area: docs
files:
  - docs/src/
---

## Problem

Phase 3 introduces first-class enums (named-field variants, match destructuring)
— a significant new language feature. The Antimony documentation (mdBook in
`docs/`) will not cover enums at all unless explicitly updated after Phase 3
builds the feature. Without docs, users of the language have no reference for:

- Enum declaration syntax: `enum Token { Ident { name: str }, Plus }`
- Unit variants (no payload): `enum Direction { North, South, East, West }`
- Match destructuring with named fields: `case Ident { name } => ...`
- How enums are represented in memory / what the compiler does with them
- Worked examples (token kind, AST node, state machine)

## Solution

After Phase 3 execution passes verification, add an "Enums" page to the mdBook
docs covering:
- Declaration syntax (named-field variants + unit variants)
- Match destructuring
- Worked example: a mini token type `enum TokenKind { Ident { name: str }, Plus, Int { value: int } }`
- Note on deferred features (positional variants)

The page should follow the style of existing language reference pages in `docs/src/`.
