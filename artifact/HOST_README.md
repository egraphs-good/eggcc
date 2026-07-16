# eggcc Artifact — Setup

This artifact is packaged as a VirtualBox VM. **This file gets you into the VM; everything
else — running the benchmarks and reproducing the paper's figures — is in the guide inside
the VM.** It supports the paper *Efficient Extraction for Effectful E-Graphs*.

## Requirements

- **An ARM host** — an Apple-Silicon Mac (M1/M2/M3/M4) or Windows on ARM — with **VirtualBox
  7.2 or newer** (free): <https://www.virtualbox.org/>. Version 7.2 is the first that runs VMs
  on ARM Macs/Windows, so anything older won't work. On a Mac: `brew install --cask virtualbox`.
  **This is an arm64 VM and will not import on Intel/x86 hosts** — if you only have an x86
  machine, contact the authors for remote access.
- A host with **≥ 8 GB RAM free** and **~40 GB free disk**. The VM is configured for 8 GB RAM
  and 4 CPUs; lower these in the VM's **Settings → System** if your host is smaller.

## Import and start

1. Import the appliance — in VirtualBox: **File → Import Appliance…**, select
   `eggcc-artifact.ova`, accept the defaults, and click **Import**.
   (Command-line equivalent: `VBoxManage import eggcc-artifact.ova`.)
2. Select the **eggcc-artifact** VM and click **Start**.
3. Log in: user **`artifact`**, password **`artifact`**.

## Then follow the in-VM guide

When the desktop loads, open **`README.md`** in your home folder — double-click it in the
Files app, or run `xdg-open ~/README.md` (or `less ~/README.md`) in a terminal. It opens with
a quick start (`./reproduce.sh smoke` / `./reproduce.sh full`) and then the full,
claim-by-claim reproduction guide. The authors' expected figures are already in `~/reference/`
for comparison.
