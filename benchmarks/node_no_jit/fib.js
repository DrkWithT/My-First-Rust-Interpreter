const fib = (n) => {
    if (n < 2) {
        return 1;
    }

    return fib(n - 1) + fib(n - 2);
};

const start_time = Date.now();
const ans = fib(29);
const elapsed_time = Date.now() - start_time;

console.log("Finished in ", elapsed_time, "ms");

if (ans == 832040) {
    process.exit(0);
} else {
    process.exit(1);
}
