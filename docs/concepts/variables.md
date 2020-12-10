# Variables

If you are familiar with some other programming language, the way Sabre handles variables won't surprise you.

To declare a variable, the `let` keyword is used.

```
// variables.sb
fn main() {
    let x = 10
    let y = 5
    println(x + y)
}
```

Run this code using the sabre CLI:

```
$ sabre build variables.sb -o variables.js
$ node variables.js
15
```
