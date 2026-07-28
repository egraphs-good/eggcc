#!/bin/bash
# Install and/or verify a Gurobi license so the eggcc nightly can auto-detect Gurobi.
#
# Gurobi is optional: the nightly runs with the free CBC solver when Gurobi is absent
# (see the "Gurobi (optional)" section of the README). Provide a license only when you
# want the Gurobi treatments and the Gurobi-specific graphs.
#
# Usage:
#   infra/setup_gurobi.sh [path/to/gurobi.lic]
#   GRB_LICENSE_FILE=path/to/gurobi.lic infra/setup_gurobi.sh
#   make gurobi-setup LICENSE=path/to/gurobi.lic
#
# With a path, the license is copied to ~/gurobi.lic (Gurobi's default search location)
# so no environment variable is needed. With no path, the script just verifies whatever
# license Gurobi already sees. In both cases it confirms Gurobi can solve a model.

set -euo pipefail

# Source license path: positional arg, else the standard GRB_LICENSE_FILE that gurobi_cl
# itself honors, else the legacy GUROBI_LICENSE_FILE as a fallback.
LICENSE_SRC="${1:-${GRB_LICENSE_FILE:-${GUROBI_LICENSE_FILE:-}}}"
DEST="$HOME/gurobi.lic"

if ! command -v gurobi_cl >/dev/null 2>&1; then
  echo "ERROR: gurobi_cl is not on your PATH."
  echo "       Install Gurobi (https://www.gurobi.com/downloads/) and add its bin/ directory"
  echo "       to PATH, then re-run this script with your license file."
  exit 1
fi

if [ -n "$LICENSE_SRC" ]; then
  if [ ! -f "$LICENSE_SRC" ]; then
    echo "ERROR: license file not found: $LICENSE_SRC"
    exit 1
  fi
  src_abs="$(readlink -f "$LICENSE_SRC")"
  dest_abs="$(readlink -f "$DEST" 2>/dev/null || true)"
  if [ "$src_abs" != "$dest_abs" ]; then
    cp "$LICENSE_SRC" "$DEST"
    echo "Copied license to $DEST (Gurobi finds this automatically)."
  else
    echo "License already installed at $DEST."
  fi
  # License files can contain sensitive credentials (e.g. WLS secrets); keep them private.
  chmod 600 "$DEST"
  echo "(To keep the license elsewhere, set GRB_LICENSE_FILE in your shell profile instead.)"
else
  echo "No license path given; verifying the license Gurobi already sees..."
fi

# Verify by solving a trivial LP -- the same code path (gurobi_cl on a model) the
# tiger extractor uses.
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/check.lp" <<'EOF'
Maximize
 x
Subject To
 c1: x <= 1
Bounds
 0 <= x <= 1
End
EOF

if gurobi_cl "$TMP/check.lp" >/dev/null 2>&1; then
  echo "SUCCESS: Gurobi is installed and licensed."
  echo "The nightly will now auto-detect Gurobi (no flag needed); use --no-gurobi to opt out."
else
  echo "ERROR: gurobi_cl could not solve a trivial model -- the license is missing or invalid."
  echo "       Diagnose with: gurobi_cl --license"
  echo "       License help:  https://support.gurobi.com/hc/en-us/articles/360040113232"
  exit 1
fi
