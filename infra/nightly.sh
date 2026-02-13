#!/bin/bash
# Thin wrapper that passes all arguments to nightly.py
# See nightly.py for the actual implementation
# Do not edit

# determine physical directory of this script
src="${BASH_SOURCE[0]}"
while [ -L "$src" ]; do
  dir="$(cd -P "$(dirname "$src")" && pwd)"
  src="$(readlink "$src")"
  [[ $src != /* ]] && src="$dir/$src"
done
MYDIR="$(cd -P "$(dirname "$src")" && pwd)"


# locally, skip rustup and tokei install
# todo idk why this doesn't work from python
if [ "$LOCAL" == "" ]; then
  rustup update
  cargo install tokei
fi

"$MYDIR/nightly.py" "$@"
