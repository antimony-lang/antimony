fn main() {
    let num = 10
    print(fib(num))
}

fn fib(n int) int { 
    if (n <= 1) {
        return n
    }

    return fib(n-1) + fib(n-2)
} 