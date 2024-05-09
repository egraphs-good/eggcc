#!/bin/zsh

if [ ! -d "$EGGCC_ROOT" ]; then
    echo "Please set \$EGGCC_ROOT to your eggcc repo root directory. Current value: $EGGCC_ROOT"
    exit 1
fi

try() {
    echo $1
    eval $1
    st=$?
    if [ $st -ne 0 ]; then
        exit $st
    fi
}
# argument 1 is input file, argument 2 is output directory
# e.g. 1=../passing/bril/mem/bubblesort.bril
#      2=passing/bril/mem
base=$(basename $1 .bril) # bubblesort
try "cargo run -r $1 --optimize-egglog=false --optimize-bril-llvm=true --run-mode=compile-bril-llvm --llvm-output-dir=$2"
try "cargo run -r $1 --optimize-egglog=false --optimize-bril-llvm=false --run-mode=compile-bril-llvm --llvm-output-dir=$2"
try "cargo run -r $1 --run-mode=dag-conversion > $2/$base.svg"
try "cargo run -r $1 --run-mode=dag-optimize > $2/${base}_egglog_opt.svg"
