#!/usr/bin/env bash
#
# Reproduce the eggcc paper's figures and drop them in a clean location.
#
# Usage:
#   artifact/reproduce.sh [smoke|full|paper] [--out-dir DIR]
#
#   smoke  (default) ~5-10 min : 3 small benchmarks, CBC solver only. Generates a coarse
#                                sanity CDF (extraction-time-cdf-smoke.pdf) to confirm the
#                                pipeline works. Does NOT overwrite the shipped figures.
#   full             ~3-4 h    : the full benchmark suite. Uses the free CBC solver, or
#                                Gurobi too if a license is installed first
#                                (infra/setup_gurobi.sh) -- auto-detected. Generates the
#                                headline CDF, the normalized performance bar charts
#                                (bril/polybench/fenwick/raytrace), and the Fenwick chart.
#   paper            hours     : the full paper configuration (100% of regions, more
#                                samples). NOT feasible on the small VM -- see README
#                                ("Note on paper-scale runs"); use a large machine.
#                                Uses Gurobi if a license is installed (infra/setup_gurobi.sh).
#
#   --out-dir DIR   copy the produced figures into DIR (the VM's ~/reproduce.sh passes
#                   --out-dir "$HOME" so figures appear at the clean top level). Default:
#                   leave them in nightly/output/paper/.
#
# The CDF plots, per regionalized e-graph, how long the fast Statewalk DP extractor takes
# vs the optimal ILP extractor. Statewalk DP is orders of magnitude faster.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

export PATH="$HOME/.cargo/bin:$PATH"
export LLVM_SYS_180_PREFIX="/usr/lib/llvm-18/"

MODE="smoke"
OUT_DIR=""
while [ $# -gt 0 ]; do
  case "$1" in
    smoke|full|paper) MODE="$1"; shift ;;
    --out-dir) OUT_DIR="${2:?--out-dir needs a directory}"; shift 2 ;;
    --out-dir=*) OUT_DIR="${1#*=}"; shift ;;
    -h|--help) sed -n '2,30p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1 (usage: $0 [smoke|full|paper] [--out-dir DIR])" >&2; exit 1 ;;
  esac
done

# Use the pinned-dependency virtualenv created by provision.sh if present.
# Put it on PATH too, so the `full`/`paper` modes -- which delegate to nightly.sh ->
# nightly.py (run via its `#!/usr/bin/env python3` shebang) and its python3 subprocesses --
# also resolve to the venv rather than the system Python (which lacks matplotlib et al.).
PY="python3"
if [ -x "$HOME/.eggcc-venv/bin/python3" ]; then
  PY="$HOME/.eggcc-venv/bin/python3"
  export PATH="$HOME/.eggcc-venv/bin:$PATH"
  export VIRTUAL_ENV="$HOME/.eggcc-venv"
fi

PAPER="$REPO_ROOT/nightly/output/paper"
FIGURES=()   # figures this run produced, to report / copy

case "$MODE" in
  smoke)
    SMOKE_DIR="$(mktemp -d)"
    trap 'rm -rf "$SMOKE_DIR"' EXIT
    cp benchmarks/passing/bril/core/fact.bril \
       benchmarks/passing/bril/core/loopfact.bril \
       benchmarks/passing/bril/core/sum-bits.bril \
       "$SMOKE_DIR"/
    echo "[smoke] profiling 3 benchmarks with the CBC solver (~5-10 min)..."
    "$PY" -c "import sys; sys.path.insert(0,'infra'); from profile import NightlyConfig, run_profile; run_profile('nightly/data', '$SMOKE_DIR', NightlyConfig(), parallel=False)"
    # Generate only the CDF, to a distinct name so the shipped full figures are untouched.
    mkdir -p "$PAPER"
    "$PY" infra/plot_cdf.py nightly/data/profile.json "$PAPER/extraction-time-cdf-smoke.pdf" --ilp-timeout-seconds 30
    FIGURES+=("$PAPER/extraction-time-cdf-smoke.pdf")
    ;;
  full)
    # No --no-gurobi: auto-detect. If a Gurobi license was installed (setup_gurobi.sh) the
    # CDF gains a Gurobi curve; otherwise it is CBC-only. Either way it is VM-scale (1% of
    # regions), so it is feasible on a few cores.
    echo "[full] running the full nightly (~3-4 h; Gurobi if a license is installed, else CBC)..."
    bash infra/nightly.sh benchmarks/passing --local
    FIGURES+=("$PAPER/extraction-time-cdf.pdf" "$PAPER/fenwick-cycles-bar-chart.pdf")
    for f in "$PAPER"/normalized-binary-perf-chart-*.pdf; do [ -f "$f" ] && FIGURES+=("$f"); done
    ;;
  paper)
    echo "[paper] running the full paper configuration (see README; needs a large machine)..."
    bash infra/nightly.sh benchmarks/passing --local --paper
    FIGURES+=("$PAPER/extraction-time-cdf.pdf" "$PAPER/fenwick-cycles-bar-chart.pdf")
    for f in "$PAPER"/normalized-binary-perf-chart-*.pdf; do [ -f "$f" ] && FIGURES+=("$f"); done
    ;;
esac

DEST="${OUT_DIR:-$PAPER}"
[ "$DEST" != "$PAPER" ] && mkdir -p "$DEST"
COPIED=()
for f in "${FIGURES[@]}"; do
  [ -f "$f" ] || continue
  [ "$DEST" != "$PAPER" ] && cp -f "$f" "$DEST/$(basename "$f")"
  COPIED+=("$DEST/$(basename "$f")")
done

echo ""
echo "================================================================"
echo " Figures are in:  $DEST/"
for f in "${COPIED[@]}"; do echo "   $(basename "$f")"; done
echo ""
if [ "$MODE" = smoke ]; then
  echo " (smoke = coarse 3-benchmark sanity CDF; the shipped full figures are untouched.)"
else
  echo " Headline: extraction-time-cdf.pdf  (CDF of ILP vs Statewalk DP extraction times)."
fi
echo "================================================================"

# Best-effort: pop the headline figure open in the VM's PDF viewer so it's impossible to miss.
if [ -n "${DISPLAY:-}" ] && command -v xdg-open >/dev/null 2>&1 && [ "${#COPIED[@]}" -gt 0 ]; then
  xdg-open "${COPIED[0]}" >/dev/null 2>&1 &
fi
