# Control Flow

Deciding whether or not to run some code depending on if a condition is true and deciding to run some code repeatedly while a condition is true are basic building blocks in most programming languages. The most common constructs that let you control the flow of execution of Sabre code are `if` expressions and loops.

## `if` Expressions

An `if` expression allows you to branch your code depending on conditions. You provide a condition and then state, "If this condition is met, run this block of code. If the condition is not met, do not run this block of code."

Here is a basic example of an `if` expression:

```
fn main() {
    let number = 3

    if number < 5 {
        println("condition was true")
    } else {
        println("condition was false")
    }
}
```

All `if` Statements start with the keyword `if`, followed by a condition. In this case, the condition checks if the number has a value less than 5. The block of code we want to execute if the condition is true is placed immediately after the condition inside curly braces.

Optionally, we can also include an `else` expression, which we chose to do here, to give the program an alternative block of code to execute should the condition evaluate to false. If you don’t provide an `else` expression and the condition is false, the program will just skip the `if` block and move on to the next bit of code.

Try running this code; You should see the following output:

```
$ sabre run main.sb
condition was true
```

Let’s try changing the value of `number` to a value that makes the condition false to see what happens:

```
let number = 7
```

Run the program again, and look at the output:

```
$ sabre run main.sb
condition was false
```

> **Note**: It's worth noting that the condition in this code must be a bool. At the current state of the project, this is not the case, but it is subject to change at any time. **TODO**: Discuss this behavior.

### Handling multiple conditions with `else if`

You can have multiple conditions by combining `if` and `else` in an `else if` expression. For example:

```
fn main() {
    let number = 6

    if number % 4 == 0 {
        println("number is divisible by 4")
    } else if number % 3 == 0 {
        println("number is divisible by 3")
    } else if number % 2 == 0 {
        println("number is divisible by 2")
    } else {
        println("number is not divisible by 4, 3, or 2")
    }
}
```

This program has four possible paths it can take. After running it, you should see the following output:

```
$ sabre run main.sb
number is divisible by 3
```

When this program executes, it checks each `if` expression in turn and executes the first body for which the condition holds true. Note that even though 6 is divisible by 2, we don’t see the output `number is divisible by 2`, nor do we see the `number is not divisible by 4, 3, or 2` text from the else block. That’s because Sabre only executes the block for the first true condition, and once it finds one, it doesn’t even check the rest.
