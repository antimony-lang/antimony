fn main() {
    print(fib(10))
}

fn fib(n: int): int { 
    if (n <= 1) {
        return n
    }

    return fib(n-1) + fib(n-2)
} 