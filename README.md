# The Sabre Programming language

[![builds.sr.ht status](https://builds.sr.ht/~garritfra/sabre/commits/ci.yml.svg)](https://builds.sr.ht/~garritfra/sabre/commits/ci.yml?)
[![docs](https://img.shields.io/badge/docs-mdBook-blue.svg)](https://garritfra.github.io/sabre/latest)

Sabre is a bullshit-free (©) programming language that gets out of your way.
It is meant to "just work", without adding unnecessary and bloated language features.

## State of this projects

Basic algorithms should compile fine. See the [examples](./examples) More sophisticated programs will not work yet. See [TODO](./TODO) for a roadmap.

The Sabre compiler emits JavaScript, and a C backend is currently in development. Backends for WASM, x86 and ARM are planned.

## Examples

```rs
// examples/fib.sb

fn main() {
    let num: int = 10
    println(fib(num))
}

fn fib(n: int) {
    if n <= 1 {
        return n
    }

    return fib(n-1) + fib(n-2)
}

// -> 55
```

## License

This software is licensed under the [Apache-2.0 license](./LICENSE).
