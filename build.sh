#!/bin/bash

set -euo pipefail

TARGET=wasm32-unknown-unknown
FOLDER=target/$TARGET/release

build() {
    binary=$1

    cargo build --target $TARGET --release
    wasm-strip $binary
    mkdir -p www/
    wasm-opt -o $binary -Oz $binary
}

build $FOLDER/snow.wasm

ls -lh www/*.wasm
