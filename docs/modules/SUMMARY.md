# Modules and Imports

Projects naturally grow over time, and digging through 10.000 lines of code in a single file can be cumbersome. By grouping related functionality and separating code with distinct features, you’ll clarify where to find code that implements a particular feature and where to go to change how a feature works.

The programs we've written so far have been in one file. As a project grows, you can organize code by splitting it into multiple modules with a clear name.

In Sabre, every file is also a module. Let's take a look at a project structure and identify its modules.

```
.
├── foo
│   ├── bar.sb
│   └── baz
│       └── module.sb
├── main.sb
└── some_logic.sb
```

As per convention, the entrypoint for this project is the `main.sb` file in the root directory.

There is a child-module called `some_logic` at the same directory-level.

Below it, there is a directory called `foo`, containing the submodule `bar`. To address the `bar` module from our entrypoint, we'd import the following:

```
import "foo/bar"
```

> **Note**: File extensions in imports are optional. Importing `foo/bar.sb` would yield the same result as importing `foo/bar`.

## Module entrypoints

In the `foo` directory, there is another directory called `baz`, containing a single file named `module.sb`. This file is treated as a special file, since it serves as the entrypoint for that module. So, instead of importing the file explicitely:

```
// main.sb
import "foo/baz/module"
```

we can simply import the module containing this file, and Sabre will import the contained `module.sb` instead.

```
// main.sb
import "foo/baz"
```

## Using imported modules

To use code defined in a separate module, we first need to import it. This is usually done at the top of the file, but it technically doesn't make a difference where in the document the import is defined. Once the module is imported, we can use the code inside it, as if it were in the current file.

Let's say we have a module named `math.sb` in the same directory as out `main.sb`, and it defines the function `add(x: int, y: int): int`. To call it in our `main.sb`, we'd do the following:

```
import "math"

fn main() {
    println(add(1, 2))
}
```

If we run `main.sb`, we should see the expected output. Sabre has imported the `add` function from the `math` module.

```
$ sabre run main.sb
3
```
