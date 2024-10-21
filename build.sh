#!/bin/bash

set -euo pipefail

TARGET=wasm32-unknown-unknown
FOLDER=target/$TARGET/release

build() {
    binary=$1
    destination=$2

    cargo build --target $TARGET --release
    wasm-strip $binary
    wasm-opt -o $destination -Oz $binary
}

build $FOLDER/snow.wasm 2021/www/snow.wasm

ls -lh 2021/www/*.wasm

