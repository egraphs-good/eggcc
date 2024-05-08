#!/bin/zsh

# Generate optimized and unoptimized LLVM IR for all passing benchmarks

for f in ../passing/**/*.bril; do
    echo $f # ../passing/bril/mem/bubblesort.bril
    output=$(dirname $(echo $f | cut -c4-)) # passing/bril/mem
    ./dump_one.sh $f $output
done
