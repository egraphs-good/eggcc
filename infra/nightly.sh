#!/bin/bash
# The primary purpose of this script is to run all the tests and upload 
# the results to the nightly-results server. It also generates some HTML
# to display the equiderivability tests in a nicely-formatted way.

# Takes no arguments, unless in LOCAL mode
# see infra/localnightly.sh for an example of how to run this script locally

echo "Beginning eggcc nightly script..."

# -x: before executing each command, print it
# -e: exit immediately upon first error
set -x -e
# if anything in a pipeline fails, fail the whole pipeline
set -o pipefail

export PATH=~/.cargo/bin:$PATH

# locally, skip rustup and tokei install
if [ "$LOCAL" == "" ]; then
  rustup update
  cargo install tokei
fi

# determine physical directory of this script
src="${BASH_SOURCE[0]}"
while [ -L "$src" ]; do
  dir="$(cd -P "$(dirname "$src")" && pwd)"
  src="$(readlink "$src")"
  [[ $src != /* ]] && src="$dir/$src"
done
MYDIR="$(cd -P "$(dirname "$src")" && pwd)"

# eggcc paths
TOP_DIR="$MYDIR/.."
RESOURCE_DIR="$MYDIR/nightly-resources"

# Nightly run directories. OUTPUT_DIR is where results go. Other temporary files can go in NIGHTLY_DIR.
NIGHTLY_DIR="$TOP_DIR/nightly"
# Output generated from data, such as graphs and the resulting report
OUTPUT_DIR="$NIGHTLY_DIR/output"
# Output for paper figures, macros
PAPER_DIR="$NIGHTLY_DIR/output/paper"
# data_dir stays even when regenerating output with --update
DATA_DIR="$TOP_DIR/nightly/data" # data from the run stored here
# we copy the data to output for archiving
OUTPUT_DATA_DIR="$OUTPUT_DIR/data" # copy the data to output
# stores llvm svgs for the website
LLVM_DIR="$DATA_DIR/llvm"
LOG_FILE="$OUTPUT_DIR/log.txt"
PROFILE_JSON="$DATA_DIR/profile.json"

# Make sure we're in the right place
cd $MYDIR
echo "Switching to nighly script directory: $MYDIR"

# Clean previous nightly run
# CAREFUL using -f
if [ "$1" == "--update" ]; then
  echo "updating front end only (output folder) due to --update flag"
  rm -rf $OUTPUT_DIR
  mkdir -p "$OUTPUT_DIR" "$PAPER_DIR"
else
  rm -rf $NIGHTLY_DIR
  # Prepare output directories
  mkdir -p "$NIGHTLY_DIR" "$NIGHTLY_DIR" "$OUTPUT_DIR" "$DATA_DIR" "$LLVM_DIR" "$PAPER_DIR"
fi



pushd $TOP_DIR

# Run profiler.
# locally, run on argument
if [ "$1" == "--update" ]; then
  echo "skipping profile.py, updating front end"
elif [ "$LOCAL" != "" ]; then
  ./infra/profile.py "$DATA_DIR" "$@" 2>&1 | tee "$LOG_FILE"
else
  export LLVM_SYS_180_PREFIX="/usr/lib/llvm-18/"
  make runtime
  # run on all benchmarks in nightly
  ./infra/profile.py "$DATA_DIR" benchmarks/passing  2>&1 | tee "$LOG_FILE"
fi

# CFGs now generated inside profile.py (removed separate generate_cfgs.py call)

# generate the plots
# needs to know what the benchmark suites are
./infra/graphs.py  "$OUTPUT_DIR" "$PAPER_DIR" "$PROFILE_JSON" benchmarks/passing 2>&1 | tee "$LOG_FILE"

# Generate latex after running the profiler (depends on profile.json)
./infra/generate_line_counts.py "$DATA_DIR" 2>&1 | tee "$LOG_FILE"

popd

# Update HTML index page.
cp "$RESOURCE_DIR"/* "$OUTPUT_DIR"

# copy data over to output
cp -r "$DATA_DIR" "$OUTPUT_DATA_DIR"

# gzip all JSON and svgs in the nightly dir
if [ "$LOCAL" == "" ]; then
  gzip "$PROFILE_JSON"
  find "$OUTPUT_DIR" -name '*.svg' -exec gzip {} +
  find "$OUTPUT_DIR" -name '*.ll' -exec gzip {} +
fi
