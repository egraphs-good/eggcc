#!/bin/bash

set -e

git clone https://github.com/Pat-Lafon/bril
cd bril/bril-rs/brillvm
git checkout llvm-18

make rt
cargo build --release
cd ../../../


mv bril/bril-rs/brillvm/target/release/main ./brilc
mv bril/bril-rs/brillvm/rt.bc ./rt.bc
rm -rf bril


