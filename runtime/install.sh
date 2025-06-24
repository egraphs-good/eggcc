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
# use nightly-2024-06-01 so that we use LLVM version 18.1.4
# this is (very close) to the version that brillvm uses (see brillvm's Cargo.toml for the inkwell dep)
cargo +nightly-2024-05-01 rustc --release -- --emit=llvm-bc
cp ./target/release/deps/runtime-*.bc ./rt.bc
cc -c rt.c -o rt.o



