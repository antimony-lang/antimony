# The Sabre Programming language

[![](https://img.shields.io/crates/v/sabre-lang.svg)](https://crates.io/crates/sabre-lang)
![Continuous integration](https://github.com/garritfra/sabre/workflows/Continuous%20integration/badge.svg?branch=master)
[![docs](https://img.shields.io/badge/docs-mdBook-blue.svg)](https://garritfra.github.io/sabre/latest)
[![Chat on Matrix](https://img.shields.io/badge/chat-on%20Matrix-green)](https://matrix.to/#/#sabre:matrix.slashdev.space?via=matrix.slashdev.space)

Sabre is a bullshit-free (©) programming language that gets out of your way.
It is meant to "just work", without adding unnecessary and bloated language features.

## State of this projects

Currently, Sabre is a general-purpose toy language. Its primary goal is to be simple and easy to understand, not to be efficient. Most algorithms should run fine, but some features may be unstable. Standard library and documentation are still incomplete. See [TODO](./TODO) for a roadmap.

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

## Documentation

Documentation is hosted on [GitHub Pages](https://garritfra.github.io/sabre).

## Installation

See [installation](https://garritfra.github.io/sabre/latest/introduction/installation.html).

## Chat on matrix

[Join here!](https://matrix.to/#/!eaupsjLNPYSluWFJOC:matrix.slashdev.space?via=matrix.slashdev.space)

## License

This software is licensed under the [Apache-2.0 license](./LICENSE).
