argc=$#

usage_exit() {
    echo "Usage: utility.sh [help | build | test | run | profile]\n\tNote: build [dev | release | profiling]\n\trun: [args...]\n\tprofile <program-name>"
    exit $1
}

handle_no_impl() {
    echo "\033[1;31mThe action $1 is not implemented :(\033[0m";
    exit 1;
}

handle_build() {
    cargo clean && cargo build --profile $1;
}

handle_profiling() {
    has_profiling_artifact=$( find ./target/profiling );

    if [[ $has_profiling_artifact -ne 0 ]]; then
        echo "\033[1;33mProfiling build not found, rebuilding...\033[0m";
        cargo clean && cargo build --profile profiling;
    fi

    samply record -s -- ./target/profiling/loxim $1;
}

handle_run() {
    cargo run -r $1;
}

if [[ $argc -lt 1 ]]; then
    usage_exit 1
fi

action="$1"

if [[ $action = "help" ]]; then
    usage_exit 0;
elif [[ $action = "build" && $argc -eq 2 ]]; then
    handle_build $2;
elif [[ $action = "profile" && $argc -eq 2 ]]; then
    handle_profiling $2
elif [[ $action = "test" ]]; then
    # TODO: use `cargo test` cmd in a handle_tests function.
    handle_no_impl $action
elif [[ $action = "run" && $argc -ge 2 ]]; then
    handle_run "${@:2}";
else
    usage_exit 1;
fi
