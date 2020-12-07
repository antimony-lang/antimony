# The Sabre Programming language

Sabre is a bullshit-free (©) programming language that gets out of your way.
Is is meant to "just work", without adding unnecessary and bloated language features.

## State of this projects

Basic algorithms like the fibonacci sequence should compile fine. More sophisticated programs will not work yet. See [TODO](./TODO) for a roadmap.

The Sabre compiler emits JavaScript, until the language has matured sufficiently. Backends for WASM, C, x86 and ARM are planned.

## Examples

```rs
// examples/fib.sb

main :: () {
    let num = 10
    return fib(num)
}

fib :: (n) {
    if n <= 1 {
        return n
    }

    return fib(n-1) + fib(n-2)
}

// -> 55
```

## License

This software is licensed under the [Apache-2.0 license](./LICENSE).
