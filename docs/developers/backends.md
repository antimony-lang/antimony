# Backends

Antimony supports three compilation backends: JavaScript, C, and QBE. QBE is the primary systems-level target.

Backend can be specified when running on building with `--target` (`-t`) option, default is `js`:

```sh
sb -t c build in.sb --out-file out
```

## Available Backends

| Target Language | Identifier     | Stability notice |
| :-------------- | :------------- | :--------------- |
| Node.js         | `js`           | mostly stable    |
| [QBE]           | `qbe`          | work in progress |
| C               | `c`            | unstable         |

[QBE]: https://c9x.me/compile
