# The command line interface

Now that you have installed Sabre, it is time to write our first program. This is a program that will simply print a string to the screen.

# Creating a project directory

Let's begin by setting up our development workspace. Sabre really doesn't care where you store the code, so feel free to choose a different directory, than the one in this example.

```
mkdir ~/sources
cd ~/sources
mkdir hello_world
cd hello_world
```

# Writing and running a program

Next, make a new source file and call it `main.sb`. Sabre files should always end with `.sb` by convention.

Now open the main.sb file you just created and enter the following code:

```
fn main() {
    println("Hello, world!")
}
```

Save the file and go back to your terminal window. Now, run the following command to compile your program:

```
sabre build main.sb -o main.js
```

> **Note:** JavaScript is the only currently supported backend for Sabre. There will be more soon.

Now, execute the compiled program with a runtime of your choice. We will be using Node.js for this example.

```
> node main.js
Hello, world!
```

You should see the string `Hello World!` on the screen. Congrats! You have officially written a Sabre Program!
