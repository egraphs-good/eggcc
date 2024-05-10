#!/bin/bash

set -e

git clone https://github.com/uwplse/bril
cd bril/bril-rs/brillvm
git checkout 2a2e3329a6721c86ceee768f6dfea5c17fb14115

make rt
cargo build --release
cd ../../../


mv bril/bril-rs/brillvm/target/release/main ./brilc
mv bril/bril-rs/brillvm/rt.bc ./rt.bc
rm -rf bril


