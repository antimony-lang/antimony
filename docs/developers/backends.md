# Backends

Antimony currently implements a JavaScript backend, but C and QBE backends are in development. WASM, ARM and x86 are planned.

Backend can be specified when running on building with `--target` (`-t`) option, default is `js`:

```sh
sb -t c build in.sb --out-file out
```

## Available Backends

| Target Language | Identifier     | Stability notice |
| :-------------- | :------------- | :--------------- |
| Node.js         | `js`           | mostly stable    |
| [QBE]           | `qbe`          | work in progess  |
| LLVM            | `llvm`         | unstable         |
| C               | `c`            | unstable         |

[QBE]: https://c9x.me/compile

LLVM also requires to enable `llvm` feature when building:

```sh
cargo build --features llvm
```
