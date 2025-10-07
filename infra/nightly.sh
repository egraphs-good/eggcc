#!/bin/bash

# haobin_nightly_test branch: modified to run the prototype extractor

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

# let time command return just wall clock time in seconds
export TIMEFORMAT='%3R'

export PATH=~/.cargo/bin:$PATH

rustup update
cargo install tokei

# determine physical directory of this script
src="${BASH_SOURCE[0]}"
while [ -L "$src" ]; do
  dir="$(cd -P "$(dirname "$src")" && pwd)"
  src="$(readlink "$src")"
  [[ $src != /* ]] && src="$dir/$src"
done
MYDIR="$(cd -P "$(dirname "$src")" && pwd)"

# Absolute directory paths
TOP_DIR="$MYDIR/.."
RESOURCE_DIR="$MYDIR/nightly-resources"
NIGHTLY_DIR="$TOP_DIR/nightly"
OUTPUT_DIR="$NIGHTLY_DIR/output"
GRAPHS_DIR="$NIGHTLY_DIR/output/graphs"
DATA_DIR="$TOP_DIR/nightly/data"

EGGLOG_DIR="$NIGHTLY_DIR/egglog"
TIGER_DIR="$NIGHTLY_DIR/tiger"

# Make sure we're in the right place
cd $MYDIR
echo "Switching to nighly script directory: $MYDIR"

# Clean previous nightly run
# CAREFUL using -f
#if [ "$1" == "--update" ]; then
#  echo "updating front end only (output folder) due to --update flag"
#  rm -rf $OUTPUT_DIR
#  mkdir -p "$OUTPUT_DIR" "$GRAPHS_DIR"
#else
#if [ "$LOCAL" == "" ]; then
  rm -rf $NIGHTLY_DIR
  # Prepare output directories
  # mkdir -p "$NIGHTLY_DIR" "$NIGHTLY_DIR/data" "$NIGHTLY_DIR/data/llvm" "$OUTPUT_DIR" "$GRAPHS_DIR"
#fi

mkdir -p "$NIGHTLY_DIR" "$NIGHTLY_DIR/data" "$OUTPUT_DIR"

# fetch and compile Egglog
if [ ! -d $EGGLOG_DIR ]; then
  # EGGLOG_VERSION="0be495630546acffbd545ba60feb9302281ce95c"
  # TODO: DO NOT USE VERY OLD EGGLOG!!!
  EGGLOG_VERSION="161a36f402b4b60d5457ce3979f10b4d95005d58"
  git clone --revision=$EGGLOG_VERSION git@github.com:egraphs-good/egglog.git "$EGGLOG_DIR"
  pushd "$EGGLOG_DIR"
  cargo build --release
  popd
  test "$EGGLOD_DIR/target/release/egglog"
fi

# fetch and compile TIGER
if [ ! -d $TIGER_DIR ]; then
  TIGER_VERSION="5e0e28fd4ec27e4b2e28fc52b284973cd399fc69"
  git clone --revision=$TIGER git@github.com:FTRobbin/tiger-prototype.git "$TIGER_DIR"
  pushd "$TIGER_DIR"
  make all
  popd
  test "$TIGER_DIR/json2egraph"
  test "$TIGER_DIR/main"
fi

pushd $TOP_DIR

#if [ "$LOCAL" == "" ]; then
#  # prepare environment for eggcc?
#  export LLVM_SYS_180_PREFIX="/usr/lib/llvm-18/"
#  make runtime
#fi

if [ "$LOCAL" != "" ]; then
  ./infra/profile.py "$DATA_DIR" "$@" 2>&1 | tee $NIGHTLY_DIR/log.txt
else
  # run on all benchmarks in nightly
  ./infra/profile.py "$DATA_DIR" benchmarks/passing  2>&1 | tee $NIGHTLY_DIR/log.txt
fi

popd

# Copy data directory to the artifact
cp -r "$NIGHTLY_DIR/data" "$OUTPUT_DIR/data"

# Copy log
cp "$NIGHTLY_DIR/log.txt" "$OUTPUT_DIR"

# gzip all JSON in the nightly dir
if [ "$LOCAL" == "" ]; then
  gzip "$OUTPUT_DIR/data/profile.json"
fi

: << 'OLDSCRIPT'
pushd $TOP_DIR

# Run profiler.
# locally, run on argument
if [ "$1" == "--update" ]; then
  echo "skipping profile.py, updating front end"
elif [ "$LOCAL" != "" ]; then
  ./infra/profile.py "$DATA_DIR" "$@" 2>&1 | tee $NIGHTLY_DIR/log.txt
else
  export LLVM_SYS_180_PREFIX="/usr/lib/llvm-18/"
  make runtime
  # run on all benchmarks in nightly
  ./infra/profile.py "$DATA_DIR" benchmarks/passing  2>&1 | tee $NIGHTLY_DIR/log.txt
fi

# Generate CFGs for LLVM after running the profiler
if [ "$1" == "--update" ]; then
  echo "skipping generate_cfgs.py"
else
  ./infra/generate_cfgs.py "$DATA_DIR/llvm" 2>&1 | tee $NIGHTLY_DIR/log.txt
fi

# generate the plots
# needs to know what the benchmark suites are
./infra/graphs.py "$OUTPUT_DIR" "$NIGHTLY_DIR/data/profile.json" benchmarks/passing 2>&1 | tee $NIGHTLY_DIR/log.txt

# Generate latex after running the profiler (depends on profile.json)
./infra/generate_line_counts.py "$DATA_DIR" 2>&1 | tee $NIGHTLY_DIR/log.txt

popd

# Update HTML index page.
cp "$RESOURCE_DIR"/* "$OUTPUT_DIR"

# Copy data directory to the artifact
cp -r "$NIGHTLY_DIR/data" "$OUTPUT_DIR/data"

# Copy log
cp "$NIGHTLY_DIR/log.txt" "$OUTPUT_DIR"

# gzip all JSON and svgs in the nightly dir
if [ "$LOCAL" == "" ]; then
  gzip "$OUTPUT_DIR/data/profile.json"
  find "$OUTPUT_DIR" -name '*.svg' -exec gzip {} +
  find "$OUTPUT_DIR" -name '*.ll' -exec gzip {} +
fi

OLDSCRIPT