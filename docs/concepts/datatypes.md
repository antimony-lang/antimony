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
$ sabre run main.sb
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
$ sabre run main.sb
1
Two
3
```
