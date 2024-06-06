#!/bin/bash

set -e

# remove rt.bc if it exists
if [ -f brillvm/rt.bc ]; then
    rm brillvm/rt.bc
fi

if [ -f brillvm/rt.bc ]; then
    rm brillvm/rt.o
fi

cd runtime
# Duplicate runtime.bc files can mess things up,
# so make sure we start from a clean slate.
cargo clean
cargo rustc --release -- --emit=llvm-bc 
cp ./target/release/deps/runtime-*.bc ./rt.bc
cc -c rt.c -o rt.o



