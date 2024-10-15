#!/bin/bash
# The primary purpose of this script is to run all the tests and upload 
# the results to the nightly-results server. It also generates some HTML
# to display the equiderivability tests in a nicely-formatted way.

# Takes a single argument- the directory of bril files
# to test (or a single bril file)

echo "Beginning eggcc nightly script..."

# -x: before executing each command, print it
# -e: exit immediately upon first error
set -x -e
# if anything in a pipeline fails, fail the whole pipeline
set -o pipefail

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
DATA_DIR="$TOP_DIR/nightly/data"

# Make sure we're in the right place
cd $MYDIR
echo "Switching to nighly script directory: $MYDIR"

# Clean previous nightly run
# CAREFUL using -f
rm -rf $NIGHTLY_DIR

# Prepare output directories
mkdir -p "$NIGHTLY_DIR/data" "$NIGHTLY_DIR/data/llvm" "$NIGHTLY_DIR/output"


pushd $TOP_DIR

# Run profiler.

# locally, run on argument
if [ "$LOCAL" != "" ]; then
  ./infra/profile.py "$@" "$DATA_DIR" 2>&1 | tee $NIGHTLY_DIR/log.txt
else
  export LLVM_SYS_180_PREFIX="/usr/lib/llvm-18/"
  make runtime
  # run on all benchmarks in nightly
  ./infra/profile.py benchmarks/passing/bril "$DATA_DIR"  2>&1 | tee $NIGHTLY_DIR/log.txt
fi

# Generate latex after running the profiler (depends on profile.json)
./infra/generate_line_counts.py "$DATA_DIR" 2>&1 | tee $NIGHTLY_DIR/log.txt

# Generate CFGs for LLVM after running the profiler
./infra/generate_cfgs.py "$DATA_DIR/llvm" 2>&1 | tee $NIGHTLY_DIR/log.txt

popd

# Update HTML index page.
cp "$RESOURCE_DIR"/* "$NIGHTLY_DIR/output"

# Copy data directory to the artifact
cp -r "$NIGHTLY_DIR/data" "$NIGHTLY_DIR/output/data"

# Copy log
cp "$NIGHTLY_DIR/log.txt" "$NIGHTLY_DIR/output"

# gzip all JSON in the nightly dir
if [ "$LOCAL" == "" ]; then
  gzip "$NIGHTLY_DIR/output/data/profile.json"
fi


# This is the uploading part, copied directly from Herbie's nightly script.
DIR="$NIGHTLY_DIR/output"
B=$(git rev-parse --abbrev-ref HEAD)
C=$(git rev-parse HEAD | sed 's/\(..........\).*/\1/')
RDIR="$(date +%s):$(hostname):$B:$C"

# Upload the artifact!
if [ "$LOCAL" == "" ]; then
  nightly-results publish --name "$RDIR" "$DIR"
fi