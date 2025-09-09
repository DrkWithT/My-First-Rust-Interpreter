"""
    faster_fib.py\n
    This is for a running time comparison of `fib.loxie` upon Loxie's VM vs. Python 3's VM. However, this algorithm caches the last two results despite being recursive.\n
"""

import time

def accumulate(a, b, it):
    if it < 1:
        return b
    else:
        return accumulate(b, a + b, it - 1)

def fib(it):
    return accumulate(1, 1, it - 2)

if __name__ == '__main__':
    pre_run_time = time.process_time_ns()
    answer = fib(40)
    running_time = time.process_time_ns() - pre_run_time

    print(f'\x1b[1;33mFinished in {running_time / 1000} microseconds\x1b[0m')

    if answer == 102334155:
        exit(0)
    else:
        exit(1)
