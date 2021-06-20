# Debugging

> **NOTE**: Currently, debugging is still nearly impossible in Antimony.

This document will give you some hints on debugging the Antimony compiler.

## Viewing the generated source code

Programs can be compiled to stdout. Use the `-o -` flag in combination with the
target backend:

```sh
cargo run -- -t js build -o - examples/fib.sb
```

Or, if Antimony is installed in your path:

```sh
sb -t js build -o - examples/fib.sb
```

