#!/bin/bash

set -e

# remove rt.bc if it exists
if [ -f brillvm/rt.bc ]; then
    rm brillvm/rt.bc
fi

cd runtime
cargo rustc --release -- --emit=llvm-bc 
mv ./target/release/deps/runtime-*.bc ./rt.bc



