handle_usage_exit() {
    echo "Usage: ./try_demos.sh [help | simple]\n\tRun non-stdin demo programs.";
    exit $1;
}

handle_simple_demos() {
    for next_prog in $1
    do
        cargo run -r -- "./demos/$next_prog.loxie";

        if [[ $? -ne 0 ]]; then
            echo "\033[1;31mFAILED on demo '$next_prog'\033[0m";
            exit 1;
        else
            echo "\033[1;32mCOMPLETED demo '$next_prog'\033[0m";
        fi
    done
}

dispatch_action() {
    argc=$#;
    action="$1";
    non_stdin_progs="dud primitives ifs simple_function print_sum iter_fib";

    if [[ $argc -lt 1 ]]; then
        handle_usage_exit 1;
    fi

    if [[ $action = "help" ]]; then
        handle_usage_exit 0;
    elif [[ $action = "simple" ]]; then
        handle_simple_demos "$non_stdin_progs";
    else
        handle_usage_exit 1;
    fi
}

dispatch_action "${@:1}"
