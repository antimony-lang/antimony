# Backends

Antimony currently implements a JavaScript backend, but a C backend is in development. WASM, ARM and x86 are planned.

Backend can be specified when running on building with `--target` (`-t`) option, default is `js`:

```sh
sb -t llvm build in.sb --out-file out
```

## Available Backends

| Target Language | Identifier     | Stability notice |
| :-------------- | :------------- | :--------------- |
| Node.js         | `js`           | mostly stable    |
| LLVM            | `llvm`         | unstable         |
| C               | `c`            | unstable         |
