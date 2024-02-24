#!/bin/bash

set -e

# do cleanup on exit. if debugging, comment out line 9
cleanup() {
  rm -r ./tmp/
}
trap cleanup EXIT

# TODO: take in file glob as command line argument
PROFILES=(tests/*.bril)

RUNMODES=("nothing" "rvsdg-roundtrip")

# create temporary directory structure necessary for bench runs
mkdir -p ./tmp/bench

# bench will benchmark a single bril file, outputting hyperfine contents to ./tmp/bench/<PROFILE_NAME>.json
# and will output the number of instructions it executed to ./tmp/bench/<PROFILE_NAME>.profile
bench() {
    # strip the file path down to just the file name
    # TODO: profile name is not unique, generate a unique output path (it will be aggregated anyway)
    PROFILE_FILE=$(basename -- "$1")
    PROFILE_NAME="${PROFILE_FILE%.*}"

    PROFILE_DIR=./tmp/bench/"$PROFILE_NAME"
    mkdir "$PROFILE_DIR"

    # loop over RUNMODES and generate a profile for each, leaving it in PROFILE_DIR
    for mode in "${RUNMODES[@]}"
    do
      echo "profiling $mode"
      
      # generate the instruction count profile by interp'ing
      cargo run --release "$1" --interp --profile-out="$PROFILE_DIR/$mode.profile" --run-mode "$mode"
      
      # run hyperfine with a warmup
      hyperfine --warmup 2 --export-json "$PROFILE_DIR/$mode.json" "cargo run --release $1 --interp --run-mode $mode"
    done
}

for p in "${PROFILES[@]}"
do
  bench "$p"
done

# aggregate all profile data into a single JSON array
python3 infra/aggregate.py > nightly/data/profile.json
