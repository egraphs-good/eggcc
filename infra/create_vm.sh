#!/usr/bin/env bash
#
# Create the eggcc artifact VM on an Apple-Silicon Mac via VBoxManage, bypassing the
# VirtualBox "New" GUI wizard (which tends to crash on the 7.2 ARM build). This only creates
# the VM shell -- arm64, EFI, a blank disk, and the Ubuntu ISO attached -- and boots you into
# the installer. Do the OS install by hand (user artifact/artifact; give the root LV the whole
# disk), then run infra/provision.sh inside the guest. See infra/BUILDING.md.
#
# Usage:
#   infra/create_vm.sh [--name NAME] [--iso PATH] [--cpus N] [--ram MB] [--disk MB] [--recreate]
#
#   --name NAME    VM name                (default: eggcc-artifact)
#   --iso PATH     Ubuntu 24.04 arm64 ISO (default: ~/Downloads/ubuntu-24.04-live-server-arm64.iso)
#   --cpus N       vCPUs                  (default: 4)
#   --ram MB       memory, MB             (default: 8192)
#   --disk MB      disk size, MB          (default: 60000)
#   --recreate     delete an existing VM of the same name first (proper unregister --delete)
set -euo pipefail

NAME="eggcc-artifact"
ISO="$HOME/Downloads/ubuntu-24.04-live-server-arm64.iso"
CPUS=4
RAM=8192
DISK=60000
RECREATE=0
while [ $# -gt 0 ]; do
  case "$1" in
    --name)     NAME="${2:?}"; shift 2 ;;
    --iso)      ISO="${2:?}"; shift 2 ;;
    --cpus)     CPUS="${2:?}"; shift 2 ;;
    --ram)      RAM="${2:?}"; shift 2 ;;
    --disk)     DISK="${2:?}"; shift 2 ;;
    --recreate) RECREATE=1; shift ;;
    -h|--help)  sed -n '2,17p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

# Locate VBoxManage (PATH, or the macOS app bundle).
VBM="$(command -v VBoxManage || true)"
[ -z "$VBM" ] && [ -x /Applications/VirtualBox.app/Contents/MacOS/VBoxManage ] \
  && VBM=/Applications/VirtualBox.app/Contents/MacOS/VBoxManage
[ -z "$VBM" ] && { echo "ERROR: VBoxManage not found -- install VirtualBox 7.2+." >&2; exit 1; }

if [ ! -f "$ISO" ]; then
  echo "ERROR: ISO not found: $ISO" >&2
  echo "Download the Ubuntu 24.04 arm64 server ISO (or pass --iso), e.g.:" >&2
  echo "  https://cdimage.ubuntu.com/releases/24.04.3/release/ubuntu-24.04.3-live-server-arm64.iso" >&2
  exit 1
fi

# Handle an existing VM of the same name.
if "$VBM" list vms | grep -q "\"$NAME\""; then
  if [ "$RECREATE" = 1 ]; then
    echo "Removing existing VM '$NAME'..."
    "$VBM" controlvm "$NAME" poweroff 2>/dev/null || true
    "$VBM" unregistervm "$NAME" --delete
  else
    echo "ERROR: a VM named '$NAME' already exists. Re-run with --recreate to replace it," >&2
    echo "       or remove it in VirtualBox (right-click -> Remove -> Delete all files)." >&2
    echo "       Do NOT delete the VM folder in Finder -- that leaves VirtualBox's config" >&2
    echo "       pointing at missing files (orphaned/inaccessible VMs)." >&2
    exit 1
  fi
fi

echo "Creating arm64 VM '$NAME' (${CPUS} vCPU / ${RAM} MB RAM / ${DISK} MB disk)..."
"$VBM" createvm --name "$NAME" --ostype Ubuntu_arm64 --register
# Ubuntu_arm64 defaults to the armv8virtual chipset + EFI firmware, so we don't set those.
# Two ARM-specific overrides are required, or the VM won't work:
#   - graphics: the ostype default (vboxvga) conflicts with the ARM memory map and fails to
#     start with "VERR_PGM_RAM_CONFLICT"; vmsvga is what ARM guests use.
#   - input: ARM has no PS/2 bus, so the default ps2 keyboard/mouse give a "keyboard failure"
#     in the guest. Use USB HID devices (which need a USB controller -- xHCI).
"$VBM" modifyvm "$NAME" --cpus "$CPUS" --memory "$RAM" --vram 128 \
  --graphicscontroller vmsvga --usb-xhci on --keyboard usb --mouse usbtablet \
  --clipboard-mode bidirectional
# SSH port-forward (host localhost:2222 -> guest:22) so a reviewer can ssh in from the host
# to paste the Gurobi key or scp a license -- the in-VM clipboard is unreliable.
#   ssh -p 2222 artifact@localhost
"$VBM" modifyvm "$NAME" --natpf1 "ssh,tcp,127.0.0.1,2222,,22"

DISK_PATH="$HOME/VirtualBox VMs/$NAME/$NAME.vdi"
"$VBM" createmedium disk --filename "$DISK_PATH" --size "$DISK"

# One SATA controller carries both the disk (port 0) and the install ISO (port 1); SATA is
# EFI-bootable on the ARM platform.
"$VBM" storagectl "$NAME" --name SATA --add sata --controller IntelAhci --portcount 2
"$VBM" storageattach "$NAME" --storagectl SATA --port 0 --device 0 --type hdd      --medium "$DISK_PATH"
"$VBM" storageattach "$NAME" --storagectl SATA --port 1 --device 0 --type dvddrive --medium "$ISO"

echo ""
echo "=========================================================================="
echo " Created VM '$NAME'. Start it and run the Ubuntu installer:"
echo "     VBoxManage startvm \"$NAME\"          # or open VirtualBox and click Start"
echo " In the installer:"
echo "   - username 'artifact', password 'artifact'"
echo "   - give the root filesystem the WHOLE disk (edit ubuntu-lv to use all free space)"
echo " After install + reboot, inside the VM:"
echo "   sudo apt-get update && sudo apt-get install -y git"
echo "   git clone https://github.com/egraphs-good/eggcc ~/eggcc"
echo "   bash ~/eggcc/infra/provision.sh --ref oflatt-gurobi-optional"
echo "=========================================================================="
