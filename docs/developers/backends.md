# Backends

Sabre currently implements a JavaScript backend, but a C backend is in development. WASM, ARM and x86 are planned.
The backend can be specified in the `Cargo.toml` file in the root of the project:

```toml
[features]
...

default = ["backend_c"]
```

If you're working on an unstable backend, you can override the backend using the `--features --no-default-features` flag of the cargo CLI:

```
cargo run --no-default-features --features backend_llvm ...
```

## Available Backends

| Target Language | Identifier     | Stability notice |
| :-------------- | :------------- | :--------------- |
| Node.js         | `backend_node` | mostly stable    |
| LLVM            | `backend_llvm` | unstable         |
| C               | `backend_c`    | unstable         |
