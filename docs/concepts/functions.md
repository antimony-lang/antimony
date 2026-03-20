# Functions

Functions are pervasive in Antimony code. You’ve already seen one of the most important functions in the language: the `main` function, which is the entry point of many programs. You've also seen the `fn` keyword, which allows you to declare new functions.

Antimony code uses snake_case as the conventional style for function and variable names. In snake case, all letters are lowercase and underscores separate words. Here’s a program that contains an example function definition:

```
fn main() {
    println("Hello, world!")
    another_function()
}

fn another_function() {
    println("Another function.")
}
```

We can call any function we’ve defined by entering its name followed by a set of parentheses. Because `another_function` is defined in the program, it can be called from inside the main function. Note that we defined `another_function` after the `main` function in the source code; we could have defined it before as well. Antimony doesn’t care where you define your functions, only that they’re defined somewhere.

## Function parameters

Functions can also be defined to have parameters, which are special variables that are part of a function’s signature. When a function has parameters, you can provide it with concrete values for those parameters. Technically, the concrete values are called arguments, but in casual conversation, people tend to use the words parameter and argument interchangeably for either the variables in a function’s definition or the concrete values passed in when you call a function.

The following rewritten version of `another_function` shows what parameters look like in Antimony:

```
fn main() {
    another_function(5)
}

fn another_function(x: int) {
    println(x)
}
```

## Return types

Functions can optionally return a value. To specify the return type, it is added to the function signature, similar to how variables and parameters do. Here's a simple example of a function that returns an integer:

```
fn add_one(x: int): int {}
```

Note that this function won't compile, since it doesn't actually return anything. Let's fix that by adding a `return` statement with an expression:

```
fn add_one(x: int): int {
    return x + 1
}
```

Now, if you call the function with `1` as its argument and read its value, you will see the computed result:

```
fn main() {
    let result = add_one(1)
    println(result)
}

fn add_one(x: int): int {
    return x + 1
}
```

```
$ sb run main.sb
2
```

# Simplified Function Syntax for Single Statements

Antimony supports a more concise syntax for functions that perform a single operation. This syntax is particularly useful for simple tasks, such as arithmetic operations, printing to the console, or returning a single expression. Instead of wrapping the function's body in curly braces, you can define the function using an equals sign (`=`) followed by the expression that constitutes the function's body.

## Syntax

The syntax for this simplified function declaration is as follows:

```
fn function_name(parameters): return_type = expression
```

This syntax removes the need for curly braces and the `return` keyword for single-statement functions, making the code cleaner and more readable.

## Examples

Below are examples demonstrating how to use this syntax:

**Defining a function that adds two numbers**:

```
fn add(x: int, y: int): int = x + y
```

**Defining a function that concatenates two strings**:

```
fn concat(a: string, b: string): string = a + b
```

# Variadic Functions

Antimony supports *variadic functions* — functions that accept a variable
number of arguments. The variadic parameter is declared with `...` before its
type and must always be the **last** parameter in the signature.

## Syntax

```
fn function_name(fixed_params, args: ...type) { ... }
```

Inside the function body the variadic parameter behaves like a regular array,
so you can iterate over it with a `for` loop.

## Examples

**A function that accepts any number of integers and prints them:**

```
fn print_numbers(values: ...int) {
    for value in values {
        println(value)
    }
}

fn main() {
    print_numbers(1, 2, 3, 4, 5)
}
```

**Mixing fixed and variadic parameters:**

```
fn log(level: int, messages: ...string) {
    for msg in messages {
        println(msg)
    }
}

fn main() {
    log(1, "Starting up", "Loading config")
}
```

The variadic parameter must come last. Placing it before other parameters is a
compile-time error.

## Backend notes

| Backend | Implementation |
|---------|---------------|
| JavaScript | Rest parameters (`...args`) — the engine collects arguments into a native array. |
| C | `...` trailing parameter — standard C variadic convention; `va_list` is used internally. |
| QBE | `...` in the QBE IL function signature; `vastart`/`vaarg` instructions manage the argument list at the machine level. |
