#!/usr/bin/env bash
#
# Build the eggcc artifact-evaluation VM on macOS with VirtualBox 7.2+.
#
#   ./build_vm.sh [--cpus N] [--ram MB] [--disk MB] [--ssh-port PORT]
#                 [--name NAME] [--ref REF] [--pregenerate full|smoke|none]
#                 [--iso-url URL] [--workdir DIR]
#
# It creates a VirtualBox VM (4 vCPUs / 8 GB RAM by default) and unattended-installs the
# Ubuntu 24.04 *Server* ISO -- VirtualBox drives the server installer's autoinstall reliably,
# whereas the desktop ISO's newer installer often fails at VBoxManage's "prepare" step. It
# then runs infra/provision.sh inside over SSH, which adds a minimal GNOME desktop (so the
# VM is graphical for viewing PDFs), installs every dependency, builds eggcc, and
# PRE-GENERATES the figures so the VM ships ready. When it finishes, export the appliance:
#   VBoxManage export eggcc-artifact -o eggcc-artifact.ova
#
# Total time is several hours, dominated by the one-time full-suite pre-generation (~3-4 h).
# If the full pre-generation runs short on memory, raise --ram (e.g. --ram 12288) for the
# build; reviewers only need the default to view figures + run smoke.
#
# ---------------------------------------------------------------------------------------
# NOTE: This is the ONE part of the artifact that was not tested by its author (the build
# environment had no VirtualBox). The provisioning it runs and the reproduction ARE tested.
# If a VBoxManage step misbehaves on your host, use the reliable manual fallback in
# infra/BUILDING.md ("Manual fallback").
# ---------------------------------------------------------------------------------------
set -euo pipefail

VM_NAME="eggcc-artifact"
EGGCC_REF="oflatt-gurobi-optional"
CPUS=4
RAM_MB=8192
DISK_MB=40960
SSH_PORT=2222
PREGENERATE="full"
ISO_URL="https://releases.ubuntu.com/24.04/ubuntu-24.04.4-live-server-amd64.iso"
WORKDIR="$HOME/eggcc-artifact-build"
VM_USER="eggcc"
VM_PASS="eggcc"

while [ $# -gt 0 ]; do
  case "$1" in
    --cpus)        CPUS="${2:?}"; shift 2 ;;
    --ram)         RAM_MB="${2:?}"; shift 2 ;;
    --disk)        DISK_MB="${2:?}"; shift 2 ;;
    --ssh-port)    SSH_PORT="${2:?}"; shift 2 ;;
    --name)        VM_NAME="${2:?}"; shift 2 ;;
    --ref)         EGGCC_REF="${2:?}"; shift 2 ;;
    --pregenerate) PREGENERATE="${2:?}"; shift 2 ;;
    --iso-url)     ISO_URL="${2:?}"; shift 2 ;;
    --workdir)     WORKDIR="${2:?}"; shift 2 ;;
    -h|--help)     sed -n '2,25p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

ISO_PATH="$WORKDIR/$(basename "$ISO_URL")"
KEY="$WORKDIR/id_eggcc"

command -v VBoxManage >/dev/null || { echo "ERROR: VBoxManage not found. Install VirtualBox 7.2+."; exit 1; }
mkdir -p "$WORKDIR"

# 1. ISO + SSH keypair -------------------------------------------------------------------
[ -f "$ISO_PATH" ] || { echo "Downloading Ubuntu ISO..."; curl -fL "$ISO_URL" -o "$ISO_PATH"; }
[ -f "$KEY" ] || ssh-keygen -t ed25519 -N "" -f "$KEY" -C "eggcc-artifact"
PUBKEY="$(cat "$KEY.pub")"

# 2. Create + configure the VM -----------------------------------------------------------
if ! VBoxManage showvminfo "$VM_NAME" >/dev/null 2>&1; then
  VBoxManage createvm --name "$VM_NAME" --ostype Ubuntu24_LTS_64 --register
  VBoxManage createhd --filename "$WORKDIR/$VM_NAME.vdi" --size "$DISK_MB"
  VBoxManage storagectl "$VM_NAME" --name SATA --add sata --controller IntelAhci
  VBoxManage storageattach "$VM_NAME" --storagectl SATA --port 0 --device 0 --type hdd \
    --medium "$WORKDIR/$VM_NAME.vdi"
fi
VBoxManage modifyvm "$VM_NAME" \
  --memory "$RAM_MB" --cpus "$CPUS" --vram 128 --graphicscontroller vmsvga \
  --ioapic on --nic1 nat --natpf1 "ssh,tcp,,${SSH_PORT},,22"

# 3. Unattended install of the base OS ---------------------------------------------------
# post-install: enable SSH and authorize our key so the host can drive provisioning.
POST_INSTALL="apt-get update && apt-get install -y openssh-server && \
  install -d -m700 -o ${VM_USER} -g ${VM_USER} /home/${VM_USER}/.ssh && \
  echo '${PUBKEY}' > /home/${VM_USER}/.ssh/authorized_keys && \
  chown ${VM_USER}:${VM_USER} /home/${VM_USER}/.ssh/authorized_keys && \
  chmod 600 /home/${VM_USER}/.ssh/authorized_keys && \
  echo '${VM_USER} ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/${VM_USER}"

echo "Installing Ubuntu unattended (this reboots the VM a couple of times)..."
VBoxManage unattended install "$VM_NAME" \
  --iso="$ISO_PATH" \
  --user="$VM_USER" --password="$VM_PASS" --full-user-name="eggcc artifact" \
  --locale=en_US --country=US --time-zone=UTC \
  --post-install-command="$POST_INSTALL" \
  --start-vm=gui

# 4. Wait for SSH, then provision over SSH (host-driven so you see the build output) ------
SSH="ssh -i $KEY -p $SSH_PORT -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null"
echo "Waiting for the guest to finish installing and come up on SSH (can take 10-20 min)..."
for _ in $(seq 1 120); do
  if $SSH "$VM_USER@127.0.0.1" true 2>/dev/null; then break; fi
  sleep 30
done
$SSH "$VM_USER@127.0.0.1" true 2>/dev/null || { echo "ERROR: could not reach the guest over SSH; see README manual fallback."; exit 1; }

echo "Provisioning eggcc inside the guest (this includes the ~3-4 h figure pre-generation)..."
scp -i "$KEY" -P "$SSH_PORT" -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  "$(dirname "${BASH_SOURCE[0]}")/provision.sh" "$VM_USER@127.0.0.1:/tmp/provision.sh"
$SSH "$VM_USER@127.0.0.1" "bash /tmp/provision.sh --ref '$EGGCC_REF' --pregenerate '$PREGENERATE'"

cat <<EOF

================================================================================
 VM "$VM_NAME" is provisioned and ready.
 Log in (user: $VM_USER / pass: $VM_PASS), open a terminal, and run:
     ./reproduce.sh smoke
 then open ~/extraction-time-cdf.pdf (and the bar charts) in the Files app.

 To produce the submission image:
     VBoxManage export "$VM_NAME" -o "$WORKDIR/$VM_NAME.ova"
================================================================================
EOF
