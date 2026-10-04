#!/bin/bash

wget -O- https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add -
sudo add-apt-repository "deb http://apt.llvm.org/jammy/ llvm-toolchain-jammy-18 main" -y

sudo apt-get update

sudo apt-get install clang-18 llvm-18 libllvm18 llvm-18-dev llvm-18-runtime

sudo apt-get install libpolly-18-dev
sudo apt-get install libzstd-dev
sudo apt-get install zlib1g-dev
# coinor-libcbc-dev: the CBC library (linked by the coin_cbc Rust crate)
# coinor-cbc: the `cbc` command-line binary that the tiger extractor shells out to
sudo apt install coinor-libcbc-dev coinor-cbc
