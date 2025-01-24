# Usage example:
# bash infra/localnightly.sh benchmarks/passing/bril/core --parallel
# First argument: path to benchmark or benchmark folder to run.
# Second argument: optional flag to run timing measurements in parallel (innacurate).

# optionally, the first argument can be --update to skip profiling and only update the front end visualizations

# -x: before executing each command, print it
# -e: exit immediately upon first error
set -x -e

# pass arguments to nightly.sh
LOCAL=1 bash infra/nightly.sh "$@"
cd nightly/output && python3 -m http.server 8002