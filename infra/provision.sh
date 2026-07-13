#!/usr/bin/env bash
#
# Provision a fresh Ubuntu 22.04 (jammy) machine into the eggcc artifact:
# installs every dependency, clones eggcc, builds it, and pre-generates the figures.
# Designed to run unattended (all apt installs use -y). Safe to re-run.
#
# This is what infra/build_vm.sh runs inside the guest. You can also run it by hand on
# any fresh Ubuntu 22.04 machine or VM:
#
#   bash provision.sh [--ref REF] [--repo URL] [--dir DIR] [--pregenerate full|smoke|none]
#
#   --ref REF          branch/tag/commit to build      (default: oflatt-gurobi-optional)
#   --repo URL         git URL to clone                (default: https://github.com/egraphs-good/eggcc)
#   --dir DIR          where to put the checkout        (default: $HOME/eggcc)
#   --pregenerate WHAT full | smoke | none : which figures to generate now (default: full)
set -euxo pipefail

EGGCC_REPO="https://github.com/egraphs-good/eggcc"
EGGCC_REF="oflatt-gurobi-optional"
EGGCC_DIR="$HOME/eggcc"
PREGENERATE="full"
while [ $# -gt 0 ]; do
  case "$1" in
    --ref)         EGGCC_REF="${2:?}"; shift 2 ;;
    --repo)        EGGCC_REPO="${2:?}"; shift 2 ;;
    --dir)         EGGCC_DIR="${2:?}"; shift 2 ;;
    --pregenerate) PREGENERATE="${2:?}"; shift 2 ;;
    -h|--help)     sed -n '2,20p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

export DEBIAN_FRONTEND=noninteractive

sudo apt-get update -y
# Base tooling: git/curl to fetch things, graphviz (`dot`) for CFGs, evince to view the
# result PDF, python for graph generation.
sudo apt-get install -y \
  git curl ca-certificates gnupg build-essential \
  graphviz evince xdg-utils \
  python3 python3-venv python3-pip

# --- Desktop environment (graphical, for viewing the result PDFs in-VM) -----------------
# build_vm.sh installs the base OS from the Ubuntu *Server* ISO (which VirtualBox can
# unattended-install reliably, unlike the desktop ISO), so we add a minimal GNOME desktop
# here and switch the VM to boot graphically. The guard skips this on a machine that already
# has a desktop (e.g. the manual desktop-ISO fallback). Best-effort: if it fails, the
# artifact still works over the CLI/SSH.
if ! command -v gnome-shell >/dev/null 2>&1; then
  sudo apt-get install -y ubuntu-desktop-minimal gdm3 \
    && sudo systemctl set-default graphical.target \
    || echo "WARNING: desktop install did not complete; the artifact still works over the CLI."
  # VirtualBox guest tools for auto screen-resize + host clipboard (best-effort; needs multiverse).
  sudo apt-get install -y virtualbox-guest-x11 || true
fi

# --- LLVM 18 toolchain (eggcc requires clang-18 / opt-18 / llvm-config) -----------------
# Mirrors install_ubuntu.sh but non-interactively, using a modern signed keyring.
curl -fsSL https://apt.llvm.org/llvm-snapshot.gpg.key \
  | sudo gpg --dearmor -o /usr/share/keyrings/llvm-snapshot.gpg
echo "deb [signed-by=/usr/share/keyrings/llvm-snapshot.gpg] http://apt.llvm.org/jammy/ llvm-toolchain-jammy-18 main" \
  | sudo tee /etc/apt/sources.list.d/llvm-18.list >/dev/null
sudo apt-get update -y
sudo apt-get install -y \
  clang-18 llvm-18 libllvm18 llvm-18-dev llvm-18-runtime \
  libpolly-18-dev libzstd-dev zlib1g-dev

# --- CBC (the free ILP solver eggcc uses by default; Gurobi is optional) ----------------
# coinor-libcbc-dev: the CBC library linked by the coin_cbc Rust crate.
# coinor-cbc:        the `cbc` CLI binary the Statewalk DP extractor (`tiger`) shells out to.
sudo apt-get install -y coinor-libcbc-dev coinor-cbc

# --- Rust toolchain ---------------------------------------------------------------------
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fi
# shellcheck disable=SC1090
source "$HOME/.cargo/env"

# --- Clone eggcc ------------------------------------------------------------------------
if [ ! -d "$EGGCC_DIR/.git" ]; then
  git clone "$EGGCC_REPO" "$EGGCC_DIR"
fi
cd "$EGGCC_DIR"
git fetch origin "$EGGCC_REF"
git checkout "$EGGCC_REF"

# tokei produces the nightly's line-count table (installed after checkout so cargo exists).
if ! command -v tokei >/dev/null 2>&1; then
  cargo install tokei --version 13.0.0 --locked
fi

# --- Python deps for graph generation (exact pinned versions, in a hidden venv) ---------
# Hidden (dot) directory so it does not clutter the reviewer's home folder.
python3 -m venv "$HOME/.eggcc-venv"
"$HOME/.eggcc-venv/bin/pip" install --upgrade pip
"$HOME/.eggcc-venv/bin/pip" install -r infra/requirements.txt

# --- Build eggcc + the bril LLVM runtime ------------------------------------------------
export LLVM_SYS_180_PREFIX=/usr/lib/llvm-18/
make runtime
cargo build --release

# --- Clean, reviewer-facing home layout -------------------------------------------------
# Home shows only: README.md (quickstart), reproduce.sh, eggcc/ (source), and the result
# PDFs after a run. Build/venv/toolchain dirs are hidden dotfiles.
cat > "$HOME/reproduce.sh" <<WRAP
#!/usr/bin/env bash
# Reproduce eggcc's figures and copy them into this (home) directory.
#   ./reproduce.sh [smoke|full]     (see eggcc/artifact/README.md)
exec "$EGGCC_DIR/artifact/reproduce.sh" --out-dir "\$HOME" "\$@"
WRAP
chmod +x "$HOME/reproduce.sh"

cat > "$HOME/README.md" <<'RM'
# eggcc artifact — quick start

Open a terminal and run one of:

    ./reproduce.sh smoke     # ~5-10 min : sanity check (3 benchmarks, CBC solver)
    ./reproduce.sh full      # ~3-4 h    : whole benchmark suite (CBC solver)

The figures are copied into THIS directory (your home folder):

  - extraction-time-cdf.pdf              CDF of ILP vs Statewalk DP extraction times (headline)
  - normalized-binary-perf-chart-*.pdf   performance bar charts (bril, polybench, fenwick, raytrace)
  - fenwick-cycles-bar-chart.pdf         the Fenwick-tree case study

(smoke writes a separate extraction-time-cdf-smoke.pdf and does not overwrite the above.)
Double-click a PDF to view it. This VM already ships with the full figures generated.

Full instructions (claim-by-claim guide, optional Gurobi, paper-scale runs) are in:
    eggcc/artifact/README.md
Source code for browsing / reuse is in:
    eggcc/
RM

# --- Pre-generate the figures so the VM ships ready to view -----------------------------
# Best-effort: a failure here does not fail provisioning (reviewers can run ~/reproduce.sh).
if [ "$PREGENERATE" != "none" ]; then
  "$EGGCC_DIR/artifact/reproduce.sh" "$PREGENERATE" --out-dir "$HOME" \
    || echo "WARNING: pre-generation ($PREGENERATE) did not finish; run ~/reproduce.sh in the VM."
fi

set +x
echo ""
echo "=========================================================================="
echo " eggcc artifact provisioned in: $EGGCC_DIR  (ref: $EGGCC_REF)"
echo " Home now contains README.md, reproduce.sh, eggcc/, and the generated figures."
echo " In the VM:  ./reproduce.sh smoke   (re-run the ~5-10 min sanity check)"
echo "=========================================================================="
