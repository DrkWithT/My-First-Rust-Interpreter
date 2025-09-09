const fib = (n) => {
    if (n < 2) {
        return n;
    }

    return fib(n - 1) + fib(n - 2);
};

const start_time = Date.now();
const ans = fib(35);
const elapsed_time = Date.now() - start_time;

console.log("Finished in ", elapsed_time, "ms\n\tResult: ", ans);

if (ans == 9227465) {
    process.exit(0);
} else {
    process.exit(1);
}
