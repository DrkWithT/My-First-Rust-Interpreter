argc=$#

usage_exit() {
    echo "Usage: utility.sh [help | build | test | run | profile]\n\tNote: build [dev | release | profiling]\n\trun: [args...]\n\tprofile <program-name>"
    exit $1
}

handle_build() {
    cargo clean && cargo build --profile $1;
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
    samply record -s -- ./target/profiling/rust_demo_2 $2;
elif [[ $action = "test" ]]; then
    # cargo test;
    echo "Not implemented :(";
    exit 1;
elif [[ $action = "run" && $argc -ge 2 ]]; then
    handle_run "${@:2}";
else
    usage_exit 1;
fi
