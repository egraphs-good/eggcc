#!/bin/zsh
try() {
    echo $1
    eval $1
    st=$?
    if [ $st -ne 0 ]; then
        exit $st
    fi
}

# Generate optimized and unoptimized LLVM IR for all passing tests & benchmarks
for subdir in "tests" "benchmarks"; do
    for f in ../../$subdir/passing/**/*.bril; do
        echo $f # ../../benchmarks/passing/bril/mem/bubblesort.bril
        output=$(dirname $(echo $f | cut -c7-)) # benchmarks/passing/bril/mem
        ./dump_one.sh $f $output
        st=$?
        if [ $st -ne 0 ]; then
            exit $st
        fi
    done
done
