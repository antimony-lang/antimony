# Structured data

When working with data, you often find yourself needing to group information together. This is where a `struct` could come into play. A _struct_, or _structure_, is a custom data type that lets you name and package together multiple related values that make up a meaningful group. If you’re familiar with an object-oriented language, a struct is like an object’s data attributes.

## Defining structs

To define a struct, we enter the keyword `struct` and name the entire struct. A struct’s name should describe the significance of the pieces of data being grouped together. Then, inside curly brackets, we define the names and types of the pieces of data, which we call fields. The following example shows a struct that stores information about a user account.

```
struct User {
    username: string
    email: string
    sign_in_count: int
    active: bool
}
```

Structs can be nested as a type inside other structs. For example, we could assign each user an address, which itself is a struct.

```
struct Address {
    street: string
    number: int
    postal_code: string
    city: string
}

struct User {
    username: string
    email: string
    address: Address
}
```

## Instantiating structs

To use a struct after we’ve defined it, we create an _instance_ of that struct by specifying concrete values for each of the fields. We create an instance by stating the name of the struct and then add curly brackets containing `key: value` pairs, where the keys are the names of the fields and the values are the data we want to store in those fields. We don’t have to specify the fields in the same order in which we declared them in the struct. In other words, the struct definition is like a general template for the type, and instances fill in that template with particular data to create values of the type. Let's use our `User` struct from a previous example and create an user called `alice`.

```
struct User {
    username: string
    email: string
    sign_in_count: int
    active: bool
}

let alice = new User {
    email: "alice@example.com"
    username: "alice"
    sign_in_count: 1
    active: true
}
```

To get a specific value from a struct, we can use dot notation. If we wanted just alice's email address, we could use `alice.email` wherever we wanted to use this value. Fields of structs can also be reassigned using the dot notation:

```
let alice = new User {
    email: "alice@example.com"
    username: "alice"
    sign_in_count: 1
    active: true
}

alice.sign_in_count = 2
```

## Struct methods

Antimony supports the concept of methods. A method can be described as a function on a struct. Let's take a look at a struct implementing a method.

```
struct User {
    first_name: string
    last_name: string

    fn full_name(): string {
        return self.first_name + " " + self.last_name
    }
}
```

Every instance of the `User` struct can now call `full_name()`. Note the usage of the word `self`. `self` is a special keyword referencing the struct instance the method was called on. Say we had the following instance of `User`, and called the `full_name()` method on it.

```
let alice = new User {
    first_name: "Jon"
    last_name: "Doe"
}

println(alice.full_name())
```

The `full_name` method described above will return the first name, a space and the last name of the user. If we run this code, we should see the expected output:

```
$ sb run main.sb
Jon Doe
```

Methods behave just like functions. They can return a value and take parameters. The only difference is the `self` keyword, which allows you to execute it on a specific instance of a struct.
