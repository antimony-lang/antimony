# The Sabre Programming language

[![](https://img.shields.io/crates/v/sabre-lang.svg)](https://crates.io/crates/sabre-lang)
![Continuous integration](https://github.com/garritfra/sabre/workflows/Continuous%20integration/badge.svg?branch=master)
[![docs](https://img.shields.io/badge/docs-mdBook-blue.svg)](https://garritfra.github.io/sabre/latest)
[![Chat on Matrix](https://img.shields.io/badge/chat-on%20Matrix-green)](https://matrix.to/#/#sabre:matrix.slashdev.space?via=matrix.slashdev.space)

Sabre is a bullshit-free (©) programming language that gets out of your way.
It is meant to "just work", without adding unnecessary and bloated language features.

## Why yet another language?

The of goal Sabre is to be a simple language that anyone - beginner and expert - can pick up and use. A "bullshit-free programming language" is of course a highly subjective opinion, and this project is my very own attempt at this. There are plenty of great programming languages out there, and Sabre is not meant to replace any of them. Currently, Sabre is just a general-purpose toy language. Its primary goal is to be simple and easy to understand, not to be efficient.

## Example

```rs
// examples/fib.sb

fn main() {
    let num = 10
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

## State of this project

Most algorithms should run fine, but some features may be unstable. Standard library and documentation are still incomplete. See the [open issues](https://github.com/garritfra/sabre/issues) for upcoming todos.

The Sabre compiler emits JavaScript for the Node.js runtime, and a C backend is currently under development. Backends for WASM and LLVM are planned.

## Documentation

Documentation is hosted [here](https://garritfra.github.io/sabre).

## Getting started

See the [installation](https://garritfra.github.io/sabre/latest/introduction/installation.html) instructions to get started.

## Chat on matrix

[Get in touch](https://matrix.to/#/!eaupsjLNPYSluWFJOC:matrix.slashdev.space?via=matrix.slashdev.space)!

## License

This software is licensed under the [Apache-2.0 license](./LICENSE).
