#!/bin/bash

set -e

git clone https://github.com/Pat-Lafon/bril
cd bril/bril-rs/brillvm
# check out Pat's fix from the llvm-18 branch
git checkout 67738d440d3516a99f9cdffb19c91540bfc66c8c

make rt
cargo build --release
cd ../../../


mv bril/bril-rs/brillvm/target/release/main ./brilc
mv bril/bril-rs/brillvm/rt.bc ./rt.bc
rm -rf bril


