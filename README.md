# The Sabre Programming language

[![builds.sr.ht status](https://builds.sr.ht/~garritfra/sabre/commits/ci.yml.svg)](https://builds.sr.ht/~garritfra/sabre/commits/ci.yml?)
[![docs](https://img.shields.io/badge/docs-mdBook-blue.svg)](https://garritfra.github.io/sabre/latest)

Sabre is a bullshit-free (©) programming language that gets out of your way.
It is meant to "just work", without adding unnecessary and bloated language features.

## State of this projects

Basic algorithms like the fibonacci sequence should compile fine. More sophisticated programs will not work yet. See [TODO](./TODO) for a roadmap.

The Sabre compiler emits JavaScript, until the language has matured sufficiently. Backends for WASM, C, x86 and ARM are planned.

## Examples

```rs
// examples/fib.sb

fn main() {
    let num = 10
    return fib(num)
}

fn fib(n) {
    if n <= 1 {
        return n
    }

    return fib(n-1) + fib(n-2)
}

// -> 55
```

## License

This software is licensed under the [Apache-2.0 license](./LICENSE).
