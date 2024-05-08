#!/bin/bash

sudo add-apt-repository "deb http://apt.llvm.org/jammy/ llvm-toolchain-jammy-18 main" -y
wget -O- https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add -

sudo apt-get update

sudo apt-get install clang-18 llvm-18

sudo apt-get install libpolly-18-dev
sudo apt-get install libzstd-dev
