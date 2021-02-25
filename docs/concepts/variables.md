# Variables

If you are familiar with some other programming language, the way Antimony handles variables won't surprise you.

To declare a variable, the `let` keyword is used. The type of the variable is infered, but can be specified explicitly.

> **Note**: Type inference currently only works when using the node-backend. For most other backends, the types need to be specified, until proper type inference is implemented.

```
// variables.sb
fn main() {
    let x = 10
    let y: int = 5
    println(x + y)
}
```

Run this code using the antimony CLI:

```
$ sb run variables.sb
15
```
