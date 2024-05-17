# -x: before executing each command, print it
# -e: exit immediately upon first error
set -x -e

# pass arguments to nightly.sh
LOCAL=1 bash infra/nightly.sh "$@"
cd nightly/output && python3 -m http.server