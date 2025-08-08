argc=$#

usage_exit() {
    echo "Usage: utility.sh [help | build | run | profile]\n\tNote: build [debug | release | prof]"
    exit $1
}

handle_build() {
    choice="$1"
    if [[ $choice = "debug" ]]; then
        cargo clean && cargo build
    elif [[ $choice = "release" ]]; then
        cargo clean && cargo build -r
    elif [[ $choice = "prof" ]]; then
        cargo clean && cargo build --profile profiling
    else
        usage_exit 1
    fi
}

if [[ $argc -lt 1 ]]; then
    usage_exit 1
fi

action="$1"

if [[ $action = "help" ]]; then
    usage_exit 0
elif [[ $action = "build" && $argc -eq 2 ]]; then
    handle_build $2
elif [[ $action = "profile" ]]; then
    samply record -s ./target/profiling/rust_demo_2
elif [[ $action = "run" && $argc -eq 2 ]]; then
    cargo run -r "$2"
else
    usage_exit 1
fi
