# Control Flow

Deciding whether or not to run some code depending on if a condition is true and deciding to run some code repeatedly while a condition is true are basic building blocks in most programming languages. The most common constructs that let you control the flow of execution of Antimony code are `if` expressions and loops.

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
$ sb run main.sb
condition was true
```

Let’s try changing the value of `number` to a value that makes the condition false to see what happens:

```
let number = 7
```

Run the program again, and look at the output:

```
$ sb run main.sb
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
$ sb run main.sb
number is divisible by 3
```

When this program executes, it checks each `if` expression in turn and executes the first body for which the condition holds true. Note that even though 6 is divisible by 2, we don’t see the output `number is divisible by 2`, nor do we see the `number is not divisible by 4, 3, or 2` text from the else block. That’s because Antimony only executes the block for the first true condition, and once it finds one, it doesn’t even check the rest.

### Value matching

Working with `if` statements with multiple `else` branches can become tedious. `match` statements provide a cleaner syntax for this case. You can compare `match` statements to `switch` in many other languages. Let's look at a very simple match statement.

```
    let x = 42

    match x {
        1 => println("x is 1")
        2 => println("x is 2")
        42 => println("The answer to the universe and everything!")
        else => println("This will not be called")
    }
```

In this example, we check the value of `x`, and execute some code based on the value. Instead of having to type `x == 1`, `x == 2` and so on, we instead provide the value only once, and decide what to do for each case. We can optionally provide a `else` case, which will be executed if no other case was triggered.

You can execute multiple statements inside a single case. A common case would be to log some debug output and then return a value.

```
fn invert(x: bool): bool {
    match x {
        true => {
            println("The value is true")
            return false
        }
        false => {
            println("The value is false")
            return true
        }
    }
}
```

Keep in mind that excessive use of this could hurt the readability of your code. Instead, you could try to outsource those statements into a function and call that instead.

## Loops

It's often useful to execute a block of code more than once. For this task, Antimony provides different kind of _loops_. A loop runs through the code inside the its body to the end and then starts immediately back at the beginning.

Antimony has two types of loops: `while` and `for`. Let's go through each of them.

### Conditional Loops with `while`

It’s often useful for a program to evaluate a condition within a loop. While the condition is true, the loop runs. When the condition ceases to be true, the program calls `break`, stopping the loop.

The example below loops three times, counting down each time, and then, after the loop, it prints another message and exits.

```
fn main() {
    let number = 3

    while number != 0 {
        println(number)

        number = number - 1
    }

    println("LIFTOFF!!!")
}
```

### Looping Through a Collection with `for`

You could use the `while` construct to loop over the elements of a collection, such as an array. For example:

```
fn main() {
    let a = [10, 20, 30, 40, 50]
    let index = 0

    while index < 5 {
        println("the value is: " + a[index])

        index += 1
    }
}
```

Here, the code counts up through the elements in the array. It starts at index `0`, and then loops until it reaches the final index in the array (that is, when `index < 5` is no longer true). Running this code will print every element in the array:

```
$ sb run main.sb
the value is: 10
the value is: 20
the value is: 30
the value is: 40
the value is: 50
```

All five array values appear in the terminal, as expected. Even though index will reach a value of 5 at some point, the loop stops executing before trying to fetch a sixth value from the array.

But this approach is error prone; we could cause the program to crash if the index length is incorrect. It's also slow, because the compiler adds runtime code to perform the conditional check on every element on every iteration through the loop.

As a more concise alternative, you can use a `for` loop and execute some code for each item in a collection. A `for` loop looks like the following:

```
fn main() {
    let a = [10, 20, 30, 40, 50]

    for element in a {
        println("the value is: " + element)
    }
}
```

When we run this code, we’ll see the same output as in the previous example. More importantly, the code is faster and less prone to errors.

For example, in the code in the previous example, if you changed the definition of the a array to have four elements but forgot to update the condition to `while index < 4`, the program would crash. Using the `for` loop, you wouldn’t need to remember to change any other code if you changed the number of values in the array.
