#!/bin/bash

set -euo pipefail

TARGET=wasm32-unknown-unknown
FOLDER=target/$TARGET/release

deploy() {
    binary=$1
    destination=$2

    wasm-strip $binary
    wasm-opt -o $destination -Oz $binary
}

cargo build --target $TARGET --release

deploy $FOLDER/snow.wasm www/2021/snow.wasm

ls -lh www/*/*.wasm

