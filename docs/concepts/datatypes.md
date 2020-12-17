# Datatypes

Sabre comes with some generic data types.

## The Integer type

The `integer` datatype represents a 4 byte decimal number.

```
fn main() {
    let sum: int = 1 + 2
    println("1 + 2 is ", sum)
}
```

```
$ sabre run main.sb
1 + 2 is 3
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
$ sabre run main.sb
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
$ sabre run main.sb
Banana
Apple
Pineapple
```
