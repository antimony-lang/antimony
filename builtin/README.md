# QBE Builtins

`builtin_qbe.c` provides three C helper functions that cannot be expressed
cleanly in QBE IL (due to variadic calls or multi-step heap allocation):

| Function | Signature | Purpose |
|---|---|---|
| `_str_concat` | `(char *a, char *b) -> char *` | Heap-allocate concatenation of two strings |
| `_int_to_str` | `(long n) -> char *` | Format integer into heap-allocated string |
| `_read_line` | `() -> char *` | Read one line from stdin into heap-allocated buffer |

## What moved to QBE IL

The following helpers are now emitted directly as QBE IL functions in the
`RUNTIME_PREAMBLE` constant in `src/generator/qbe.rs`, so no C file is needed
for them:

| Function | Libc call | Notes |
|---|---|---|
| `_printf(msg: l)` | `write(1, msg, strlen(msg))` | Non-variadic stdout write |
| `_exit(code: w)` | `fflush(0)` + `_Exit(code)` | Flush streams, hard-exit |
| `_strlen(s: l): w` | `strlen(s)` | Word-width wrapper |
| `_parse_int(s: l): w` | `atoi(s)` | Integer parse |

## Adding a new backend

A new backend needs to provide at minimum `_str_concat`, `_int_to_str`, and
`_read_line` (or equivalent logic), since these cannot be expressed as a
single non-variadic libc call. Everything else can call libc directly.
