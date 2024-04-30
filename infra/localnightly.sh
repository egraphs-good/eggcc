# pass arguments to nightly.sh
LOCAL=1 bash infra/nightly.sh "$@"
cd nightly/output && python3 -m http.server