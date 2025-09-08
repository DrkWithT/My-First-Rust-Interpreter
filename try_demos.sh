OK_STATUS=0;
FAIL_STATUS=1;

handle_usage_exit() {
    echo "Usage: ./try_demos.sh [help | demo]\n\tdemo [simple | negatives]: Run non-stdin demo programs. 0 -> positive checks, 1 -> negative checks";
    exit $1;
}

handle_simple_demos() {
    check_status=$((0));

    if [[ "$1" = "negatives" ]]; then
        check_status=$((1));
    fi

    demos=$( find -f ./demos/$1/*.loxie );

    for next_prog in $demos
    do
        cargo run -r -- "$next_prog";

        if [[ $? -ne $check_status ]]; then
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

    if [[ $argc -lt 1 ]]; then
        handle_usage_exit 1;
    fi

    if [[ $action = "help" ]]; then
        handle_usage_exit 0;
    elif [[ $action = "demo" && $argc -eq 2 ]]; then
        handle_simple_demos "$2";
    else
        handle_usage_exit 1;
    fi
}

dispatch_action "${@:1}"
