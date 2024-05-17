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

export PATH=~/.cargo/bin:$PATH

rustup update

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

# Make sure we're in the right place
cd $MYDIR
echo "Switching to nighly script directory: $MYDIR"

# Clean previous nightly run
# CAREFUL using -f
rm -rf $NIGHTLY_DIR

# Prepare output directories
mkdir -p "$NIGHTLY_DIR/data" "$NIGHTLY_DIR/output"


pushd $TOP_DIR

# Run profiler.
# create temporary directory structure necessary for bench runs
mkdir -p ./tmp/bench

export LLVM_SYS_180_PREFIX="/usr/lib/llvm-18/"

# locally, run on argument
if [ "$LOCAL" != "" ]; then
  ./infra/profile.py "$@"
else
  # run on all benchmarks in nightly
  ./infra/profile.py benchmarks/passing
fi


rm -r ./tmp/

popd


# Update HTML index page.
cp "$RESOURCE_DIR"/* "$NIGHTLY_DIR/output"

# Copy json directory to the artifact
cp -r "$NIGHTLY_DIR/data" "$NIGHTLY_DIR/output/data"

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