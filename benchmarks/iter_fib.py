"""
    iter_fib.py\n
    This is for a running time comparison against `iter_fib.loxie` upon the Conch VM.
"""

import time

def iter_fib(n):
    it = n
    temp = 0
    a = 0
    b = 1

    while it > 0:
        temp = a
        a = b
        b = temp + b
        it -= 1

    return b

if __name__ == '__main__':
    pre_run_time = time.process_time_ns()
    answer = iter_fib(39)
    running_time = time.process_time_ns() - pre_run_time

    print(f'\x1b[1;33mFinished in {running_time / 1000} microseconds\x1b[0m')

    if answer == 102334155:
        exit(0)
    else:
        exit(1)
