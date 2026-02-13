# Usage example:
# bash infra/localnightly.sh benchmarks/passing/bril/core --parallel
# First argument: path to benchmark or benchmark folder to run.
# Additional arguments: optional flags like --parallel, --update, --paper, etc.
#
# This script runs nightly.sh in local mode (skips rustup update and tokei install)
# and then serves the results on http://localhost:8002

# -x: before executing each command, print it
# -e: exit immediately upon first error
set -x -e

# pass arguments to nightly.sh with --local flag
bash infra/nightly.sh --local "$@"
cd nightly/output && python3 -m http.server 8002