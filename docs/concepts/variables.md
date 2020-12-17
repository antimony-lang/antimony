# Variables

If you are familiar with some other programming language, the way Sabre handles variables won't surprise you.

To declare a variable, the `let` keyword is used. The type of the variable is infered, but can be specified explicitly.

```
// variables.sb
fn main() {
    let x = 10
    let y: int = 5
    println(x + y)
}
```

Run this code using the sabre CLI:

```
$ sabre run variables.sb
15
```
