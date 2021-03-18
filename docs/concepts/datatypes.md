# Datatypes

Antimony comes with some generic data types.

## The Boolean type

As in most other programming languages, a Boolean type in Antimony has two possible values: `true` and `false`. Booleans are one byte in size. The Boolean type in Antimony is specified using `bool`. For example:

```
fn main() {
    let t = true
    let f: bool = false // with explicit type annotation
}
```

The main way to use Boolean values is through conditionals, such as an `if` expression. We’ll cover how `if` expressions work in the ["Control Flow"](introduction/control-flow.md) section.

## The Integer type

The `integer` datatype represents a 4 byte decimal number.

```
fn main() {
    let sum: int = 1 + 2
    println("1 + 2 is ", sum)
}
```

```
$ sb run main.sb
1 + 2 is 3
```

Decimal, binary, hexadecimal and octal number systems are supported. The number `255` can be written in these formats:

```
let binary = 0b11111111
let octal = 0o37
let decimal = 255
let hexadecimal = 0xFF
```

To make large numbers more readable, you can insert `_` characters at arbitrary places. These characters will be ignored by the compiler.

```
let one_billion = 1_000_000_000
```

## The String type

A string is a sequence of characters.

```
fn main() {
    let name: string = "Jon"
    println("Hello " + name)
}
```

```
$ sb run main.sb
Hello Jon
```

## The Array type

Arrays represent a sequence of values. They can hold any number of values of a specific type.

```
fn main() {
    let fruits: string[] = ["Banana", "Apple", "Pineapple"]

    for fruit in fruits {
        println(fruit)
    }
}
```

```
$ sb run main.sb
Banana
Apple
Pineapple
```

Arrays have a fixed capacity. In most cases, the capacity of an array can be infered. In the example above, the compiler knows that three elements are in the array, so it can be inferred. If the capacity can't be inferred by the compiler, it is necessary to mark it explicitely. This is the case for uninitialized arrays:

```
let arr: int[3]
arr[0] = 1
arr[1] = 2
arr[2] = 3

for element in arr {
    println(element)
}
```

## The Any type

`any` can be used to specify that any type can be used in this place. This should be used with caution, as it might cause undefined behavior.

```
fn main() {

    print_anything(5)
    print_anything("Hello")
}

fn print_anything(x: any) {
    println(x)
}
```

```
$ sb run main.sb
5
Hello
```

`any` can also be used in conjunction with the array notation to allow a mixture of types within an array.

```
fn main() {

    let arr = [1, "Two", 3]

    for x in arr {
        println(x)
    }
}
```

```
$ sb run main.sb
1
Two
3
```
