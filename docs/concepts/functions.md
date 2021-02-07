# Functions

Functions are pervasive in Sabre code. You’ve already seen one of the most important functions in the language: the `main` function, which is the entry point of many programs. You've also seen the `fn` keyword, which allows you to declare new functions.

Sabre code uses snake_case as the conventional style for function and variable names. In snake case, all letters are lowercase and underscores separate words. Here’s a program that contains an example function definition:

```
fn main() {
    println("Hello, world!")
    another_function()
}

fn another_function() {
    println("Another function.")
}
```

We can call any function we’ve defined by entering its name followed by a set of parentheses. Because `another_function` is defined in the program, it can be called from inside the main function. Note that we defined `another_function` after the `main` function in the source code; we could have defined it before as well. Sabre doesn’t care where you define your functions, only that they’re defined somewhere.

## Function parameters

Functions can also be defined to have parameters, which are special variables that are part of a function’s signature. When a function has parameters, you can provide it with concrete values for those parameters. Technically, the concrete values are called arguments, but in casual conversation, people tend to use the words parameter and argument interchangeably for either the variables in a function’s definition or the concrete values passed in when you call a function.

The following rewritten version of `another_function` shows what parameters look like in Sabre:

```
fn main() {
    another_function(5)
}

fn another_function(x: int) {
    println(x)
}
```

## Functions contain statements

TODO

## Return types

TODO
