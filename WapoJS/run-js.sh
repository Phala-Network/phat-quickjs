#!/bin/bash

set -e

RUNNER=./runner/target/release/wapojs-run
ENGINE=./wapojs.wasm

# if engine doesn't exist, build it
if [ ! -f $ENGINE ]; then
    echo "Building engine..."
    make
fi
# if runner doesn't exist, build it
if [ ! -f $RUNNER ]; then
    echo "Building runner..."
    cd runner
    cargo build --release
    cd ..
fi

REST_ARGS="${@:2}"
$RUNNER --engine $ENGINE $1 -- $REST_ARGS
 