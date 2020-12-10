# Datatypes

> **Note:** Due to the fact that Sabre currently emits javascript, static types are not needed, as of yet. They will however be introduced soon, once a statically typed backend is used.

## The Integer type

The `integer` datatype represents a number. The JavaScript backend interprets any integer as a `Number` type.

```
fn main() {
    let sum = 1 + 2
    println("1 + 2 is ", sum)
}
```

```
$ sabre build main.sb -o main.js
$ node main.js
1 + 2 is 3
```

## The String type

A string is a sequence of characters.

```
fn main() {
    let name = "Jon"
    println("Hello " + name)
}
```

```
$ sabre build main.sb -o main.js
$ node main.js
Hello Jon
```

## The Array type

Arrays represent a sequence of values. They can hold any number of values of a specific type.

> **NOTE:** Currently, there is no type-checking involved when creating arrays. There will be, once a type system is in place, so don't get too attached to mixing and matching element types. ;)

```
fn main() {
    let fruits = ["Banana", "Apple", "Pineapple"]

    for fruit in fruits {
        println(fruit)
    }
}
```

```
$ sabre build main.sb -o main.js
$ node main.js
Banana
Apple
Pineapple
```
