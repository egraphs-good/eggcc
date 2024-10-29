# smalls script to test changes to the javascript or html files
# determine physical directory of this script
src="${BASH_SOURCE[0]}"
while [ -L "$src" ]; do
  dir="$(cd -P "$(dirname "$src")" && pwd)"
  src="$(readlink "$src")"
  [[ $src != /* ]] && src="$dir/$src"
done
MYDIR="$(cd -P "$(dirname "$src")" && pwd)"
TOP_DIR="$MYDIR/.."
RESOURCE_DIR="$MYDIR/nightly-resources"
NIGHTLY_DIR="$TOP_DIR/nightly"

echo "Copying resources to $NIGHTLY_DIR/output"
cp "$RESOURCE_DIR"/* "$NIGHTLY_DIR/output"

cd nightly/output && python3 -m http.server 8002