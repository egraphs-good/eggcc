#!/usr/bin/env bash
#
# Provision a fresh Ubuntu 24.04 (noble) machine into the eggcc artifact:
# installs every dependency, clones eggcc, builds it, and pre-generates the figures.
# Designed to run unattended (all apt installs use -y). Safe to re-run.
#
# Run it inside a fresh Ubuntu 24.04 machine or VM (see infra/BUILDING.md):
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

# --- Self-heal a stale VM clock before touching apt -------------------------------------
# A VM restored from an old snapshot/export can boot with its clock in the past, which makes
# apt reject repo metadata as "not valid yet" (e.g. "<suite>/InRelease is not valid yet") and
# aborts provisioning at the first apt-get update. Enable NTP and give it a moment to correct.
# Best-effort: this never fails provisioning (if there is no network to sync from, we proceed
# and apt will surface the real problem).
if command -v timedatectl >/dev/null 2>&1; then
  sudo timedatectl set-ntp true || true
  sudo systemctl restart systemd-timesyncd 2>/dev/null || true
  # Wait up to ~30s for the clock to actually sync before proceeding.
  for _ in $(seq 1 15); do
    if [ "$(timedatectl show -p NTPSynchronized --value 2>/dev/null)" = "yes" ]; then
      break
    fi
    sleep 2
  done
fi

# NOTE: disk sizing is intentionally left to the installer / operator. Ubuntu Server's guided
# LVM allocates only ~half the volume group to the root LV by default, which is too small for
# the LLVM build + cargo target + Gurobi + a full run -- so when creating the VM, give the
# root filesystem the whole disk (in the installer, edit ubuntu-lv to use all free space; or
# afterwards: sudo lvextend -l +100%FREE /dev/ubuntu-vg/ubuntu-lv && sudo resize2fs /dev/ubuntu-vg/ubuntu-lv).

# The installer registers the install ISO as an apt source (a `cdrom:`/`file:/cdrom` line).
# Once the ISO is ejected, `apt-get update` fails on it ("no longer has a Release file"), so
# disable any cdrom apt source before updating -- keeps apt clean whether the ISO is attached
# or not, including in the shipped VM.
sudo sed -i '/cdrom/ s/^[^#]/# &/' /etc/apt/sources.list 2>/dev/null || true

sudo apt-get update -y
# Base tooling: git/curl to fetch things, graphviz (`dot`) for CFGs, evince to view the
# result PDF, python for graph generation, openssh-server so a reviewer can ssh in from the
# host (to paste the Gurobi key or scp a license -- the VM clipboard is unreliable).
sudo apt-get install -y \
  git curl ca-certificates gnupg build-essential \
  graphviz evince xdg-utils openssh-server \
  python3 python3-venv python3-pip

# --- Desktop environment (graphical, for viewing the result PDFs in-VM) -----------------
# The base OS is usually installed from the Ubuntu *Server* ISO, so we add a minimal GNOME
# desktop here and switch the VM to boot graphically. The guard skips this on a machine that
# already has a desktop (e.g. if you installed from the Desktop ISO). Best-effort: if it
# fails, the artifact still works over the CLI/SSH.
if ! command -v gnome-shell >/dev/null 2>&1; then
  # Keep the Firefox snap OUT of the desktop install: seeding it (a large squashfs + apparmor)
  # reliably crashes the VirtualBox 7.2 ARM preview during provisioning. The artifact needs no
  # browser anyway -- the default CBC path opens none, and Gurobi licensing uses `grbgetkey` in
  # a terminal. Pinning the transitional `firefox` deb out stops ubuntu-desktop-minimal (where
  # it is only a Recommends) from pulling the snap.
  printf 'Package: firefox*\nPin: release o=*\nPin-Priority: -1\n' \
    | sudo tee /etc/apt/preferences.d/no-firefox-snap >/dev/null
  sudo apt-get install -y ubuntu-desktop-minimal gdm3 \
    && sudo systemctl set-default graphical.target \
    || echo "WARNING: desktop install did not complete; the artifact still works over the CLI."
  # VirtualBox guest tools for auto screen-resize + host clipboard (best-effort; needs multiverse).
  sudo apt-get install -y virtualbox-guest-x11 || true
fi

# Configure the desktop whenever one is present -- this runs both on the fresh install above
# AND on a re-provision of an existing VM. (The install block is guarded on gnome-shell being
# ABSENT, so this config must live outside it, or re-runs would silently skip it.)
if command -v gnome-shell >/dev/null 2>&1; then
  # gnome-terminal because ubuntu-desktop-minimal may omit it, and the quick start tells
  # reviewers to "open a terminal".
  sudo apt-get install -y gnome-terminal || true

  # System dconf defaults for the reviewer's desktop (applied at login; no graphical session
  # needed at provision time, unlike `gsettings`):
  #   - never blank/lock the screen or auto-suspend, so a long reproduce.sh run doesn't drop
  #     the reviewer to the login screen;
  #   - pin the apps the artifact actually uses to the dock (Terminal first, then Files for
  #     browsing ~/reference, Evince for the PDFs). Unknown/uninstalled .desktop ids are
  #     simply ignored by GNOME.
  sudo mkdir -p /etc/dconf/profile /etc/dconf/db/local.d
  if [ ! -f /etc/dconf/profile/user ]; then
    printf 'user-db:user\nsystem-db:local\n' | sudo tee /etc/dconf/profile/user >/dev/null
  fi
  sudo tee /etc/dconf/db/local.d/00-eggcc-desktop >/dev/null <<'DCONF'
[org/gnome/desktop/session]
idle-delay=uint32 0

[org/gnome/desktop/screensaver]
lock-enabled=false
idle-activation-enabled=false

[org/gnome/settings-daemon/plugins/power]
sleep-inactive-ac-type='nothing'
sleep-inactive-battery-type='nothing'

[org/gnome/shell]
favorite-apps=['org.gnome.Terminal.desktop', 'org.gnome.Nautilus.desktop', 'org.gnome.Evince.desktop']
DCONF
  # Remove the old filename from earlier builds so it doesn't linger with stale settings.
  sudo rm -f /etc/dconf/db/local.d/00-eggcc-no-idle
  sudo dconf update || true
fi

# Don't block boot for ~2 minutes on the network. Once the desktop's NetworkManager manages the
# interface, systemd-networkd-wait-online never sees "all links online" and waits its full
# timeout at every boot. Mask both wait-online units -- networking still comes up normally, boot
# just doesn't stall on it. (Safe/no-op if a unit isn't present.)
sudo systemctl mask systemd-networkd-wait-online.service 2>/dev/null || true
sudo systemctl mask NetworkManager-wait-online.service 2>/dev/null || true

# --- LLVM 18 toolchain (eggcc requires clang-18 / opt-18 / llvm-config) -----------------
# Ubuntu 24.04 (noble) ships LLVM 18 in its own 'universe' repo (enabled by default), so we
# install from there. This deliberately avoids the external apt.llvm.org repo, which has
# repeatedly flaked (DNS "could not resolve host apt.llvm.org") mid-provision.
# Remove any stale apt.llvm.org source left by an earlier build -- otherwise its snapshot
# packages clash with Ubuntu's (e.g. "libllvm18 Breaks llvm-18-dev"). Refresh apt afterwards.
sudo rm -f /etc/apt/sources.list.d/llvm-18.list /usr/share/keyrings/llvm-snapshot.gpg
sudo apt-get update -y
sudo apt-get install -y \
  clang-18 llvm-18 libllvm18 llvm-18-dev llvm-18-runtime \
  libpolly-18-dev libzstd-dev zlib1g-dev

# --- CBC (the free ILP solver eggcc uses by default; Gurobi is optional) ----------------
# coinor-libcbc-dev: the CBC library linked by the coin_cbc Rust crate.
# coinor-cbc:        the `cbc` CLI binary the Statewalk DP extractor (`tiger`) shells out to.
sudo apt-get install -y coinor-libcbc-dev coinor-cbc

# --- Gurobi solver (optional; reviewers add their own license) --------------------------
# eggcc defaults to CBC; Gurobi is optional. We install the *solver* here so a reviewer only
# has to add their own license (README "Optional -- Gurobi"). The license is NOT installed or
# shipped. Best-effort: a download/arch failure never fails provisioning (CBC still works).
# Override the version with GUROBI_VERSION=... if needed.
GUROBI_VERSION="${GUROBI_VERSION:-12.0.1}"
if ! command -v gurobi_cl >/dev/null 2>&1; then
  case "$(uname -m)" in
    x86_64)  GRB_ARCH="linux64" ;;
    aarch64) GRB_ARCH="armlinux64" ;;
    *)       GRB_ARCH="" ;;
  esac
  if [ -z "$GRB_ARCH" ]; then
    echo "WARNING: no Gurobi build for arch $(uname -m); skipping Gurobi (CBC still works)."
  else
    grb_short="${GUROBI_VERSION%.*}"                    # 12.0.1 -> 12.0
    grb_tar="gurobi${GUROBI_VERSION}_${GRB_ARCH}.tar.gz"
    grb_dir="gurobi$(echo "$GUROBI_VERSION" | tr -d .)" # 12.0.1 -> gurobi1201
    if curl -fsSL "https://packages.gurobi.com/${grb_short}/${grb_tar}" -o "/tmp/${grb_tar}"; then
      sudo tar -xzf "/tmp/${grb_tar}" -C /opt
      rm -f "/tmp/${grb_tar}"
      # Put gurobi_cl on PATH for all login shells (the tiger extractor shells out to it).
      sudo tee /etc/profile.d/gurobi.sh >/dev/null <<PROF
export GUROBI_HOME=/opt/${grb_dir}/${GRB_ARCH}
export PATH="\$GUROBI_HOME/bin:\$PATH"
PROF
      export GUROBI_HOME="/opt/${grb_dir}/${GRB_ARCH}"
      export PATH="$GUROBI_HOME/bin:$PATH"
      echo "Installed Gurobi ${GUROBI_VERSION} (${GRB_ARCH}) to /opt/${grb_dir}."
    else
      echo "WARNING: could not download Gurobi ${GUROBI_VERSION} (${GRB_ARCH}); skipping (CBC still works)."
    fi
  fi
fi

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

# One README at the home top level: the artifact guide, which opens with a quick start.
cp "$EGGCC_DIR/artifact/README.md" "$HOME/README.md"

# --- Pre-generate the reference figures so the VM ships ready to view -------------------
# These go in ~/reference (the untouched "answer key"); the reviewer's own reproduce.sh runs
# write alongside them in the home folder for comparison. Best-effort: a failure here does
# not fail provisioning (reviewers can still run ~/reproduce.sh). Pass --pregenerate none to
# skip this (e.g. when you hand-curate ~/reference for the shipped image).
if [ "$PREGENERATE" != "none" ]; then
  mkdir -p "$HOME/reference"
  "$EGGCC_DIR/artifact/reproduce.sh" "$PREGENERATE" --out-dir "$HOME/reference" \
    || echo "WARNING: pre-generation ($PREGENERATE) did not finish; run ~/reproduce.sh in the VM."
fi

set +x
echo ""
echo "=========================================================================="
echo " eggcc artifact provisioned in: $EGGCC_DIR  (ref: $EGGCC_REF)"
echo " Home now contains README.md (quick start at top), reproduce.sh, eggcc/, and"
echo " reference/ (the pre-generated figures). Reviewer runs write figures into the home folder."
echo " In the VM:  ./reproduce.sh smoke   (re-run the ~5-10 min sanity check)"
echo "=========================================================================="
