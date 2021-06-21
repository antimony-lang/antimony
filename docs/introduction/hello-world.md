# The command line interface

Now that you have installed Antimony, it is time to write our first program. This is a program that will simply print a string to the screen.

# Creating a project directory

Let's begin by setting up our development workspace. Antimony really doesn't care where you store the code, so feel free to choose a different directory, than the one in this example.

```
mkdir ~/sources
cd ~/sources
mkdir hello_world
cd hello_world
```

# Writing and running a program

Next, make a new source file and call it `main.sb`. Antimony files should always end with `.sb` by convention.

Now open the main.sb file you just created and enter the following code:

```
pub fn main() {
    println("Hello, world!")
}
```

Save the file and go back to your terminal window. Now, run the following command to compile and run your program:

```
$ sb run main.sb
```

You should see the string `Hello World!` on the screen. Congrats! You have officially written a Antimony Program!
