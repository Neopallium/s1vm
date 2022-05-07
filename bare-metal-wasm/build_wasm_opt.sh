#!/bin/bash

set -euo pipefail

TARGET=wasm32-unknown-unknown
BINARY=target/$TARGET/release/bare_metal_wasm.wasm
OUT=bare_metal_wasm.wasm

cargo build --target $TARGET --release
wasm-strip $BINARY
wasm-opt -o $OUT -Oz $BINARY
ls -lh $OUT
