# Backends

Sabre currently implements a JavaScript backend, but a C backend is in development. WASM, ARM and x86 are planned.
The backend can be specified in the `Cargo.toml` file in the root of the project:

```toml
[features]
...

default = ["backend_c"]
```

## Available Backends

| Target Language | Identifier     | Stability notice      |
| :-------------- | :------------- | :-------------------- |
| Node.js         | `backend_node` | mostly stable         |
| C               | `backend_c`    | in active development |
