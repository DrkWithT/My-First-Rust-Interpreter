"""
    fib.py\n
    This is for a running time comparison against `fib.loxie` upon the Conch VM.\n
"""

import time

def fib(n):
    """
        This uses the naive recursive method for finding the n-th Fibonacci term like the corresponding Loxie program.\n
    """
    if n < 2:
        return 1
    else:
        return fib(n - 1) + fib(n - 2)

if __name__ == '__main__':
    pre_run_time = time.process_time_ns()
    answer = fib(39)
    running_time = time.process_time_ns() - pre_run_time

    print(f'\x1b[1;33mFinished in {running_time / 1000000}ms\x1b[0m')

    if answer == 102334155:
        exit(0)
    else:
        exit(1)
