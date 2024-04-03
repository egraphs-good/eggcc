#!/bin/bash

set -e

# do cleanup on exit. if debugging, comment out line 9
cleanup() {
  rm -r ./tmp/
}
trap cleanup EXIT

# TODO: take in file glob as command line argument
PROFILES=(tests/*.bril tests/small/*.bril tests/brils/passing/**/*.bril)

# create temporary directory structure necessary for bench runs
mkdir -p ./tmp/bench

# Benchmark a single bril file
# Use Brilift to compile to an executable, then use hyperfine to benchmark the runtime
# Outputs the hyperfine results to "tmp/bench/<BRIL_NAME>/brilift.json"
bench() {
  # strip the file path down to just the file name
  # TODO: profile name is not unique, generate a unique output path (it will be aggregated anyway)
  PROFILE_FILE=$(basename -- "$1")
  PROFILE_NAME="${PROFILE_FILE%.*}"

  PROFILE_DIR=./tmp/bench/"$PROFILE_NAME"
  mkdir "$PROFILE_DIR"

  # Run eggcc in compile-brilift mode, which just shells out to brilift to compile the bril file
  cargo run --release "$1" --run-mode compile-brilift -o "$PROFILE_DIR/brilift"
  
  # TODO: Some of the brils result in executables that seg fault, need to figure out why
  # For now, though, we just silently ignore them
  hyperfine --warmup 2 --export-json "$PROFILE_DIR"/brilift.json "$PROFILE_DIR/brilift" || echo "[BRILIFT] could not run $PROFILE_NAME"
}

for p in "${PROFILES[@]}"
do
  bench "$p"
done

python3 infra/aggregate.py > nightly/data/profile.json