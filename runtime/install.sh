#!/bin/bash

echo "Building runtime.bc and rt.o"

set -e

# remove rt.bc if it exists
if [ -f runtime/rt.bc ]; then
    rm runtime/rt.bc
fi

if [ -f runtime/rt.bc ]; then
    rm runtime/rt.o
fi

cd runtime
# Duplicate runtime.bc files can mess things up,
# so make sure we start from a clean slate.
cargo clean
cargo rustc --release -- --emit=llvm-bc 
cp ./target/release/deps/runtime-*.bc ./rt.bc
cc -c rt.c -o rt.o



