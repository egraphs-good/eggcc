# Building and shipping the eggcc artifact VM

Author-facing guide for building `eggcc-artifact.ova` by hand on an Apple-Silicon Mac.
Reviewers don't need this — they import and run the VM following `artifact/README.md`, the
single unified guide: its §0 covers host-side import/boot (shipped next to the OVA), and the
rest is the in-VM reproduction guide (also at `~/README.md` inside the VM).

The artifact ships as an **arm64** VM (native on Apple-Silicon Macs and Windows on ARM; it will
not import on x86 hosts — see `artifact/README.md` §0).

## 0. Prerequisites

- **Push the branch to GitHub first.** `provision.sh` clones eggcc from GitHub at `--ref`
  (default `oflatt-gurobi-optional`), so the `artifact/` and `infra/` scripts must be on that
  branch *on GitHub* before you build. (Once the code lands on `main`, pass `--ref main` or a tag.)
- **VirtualBox 7.2+** on the Mac — 7.2 is the first version that runs VMs on Apple Silicon:
  `brew install --cask virtualbox`.
- **Ubuntu 24.04 arm64 ISO**, e.g.
  `https://cdimage.ubuntu.com/releases/24.04.3/release/ubuntu-24.04.3-live-server-arm64.iso`
  (the Server image; `provision.sh` installs the GNOME desktop on top). The Desktop arm64 image
  works too if you'd rather install graphically. **Use arm64** — an amd64 guest runs under x86
  emulation on Apple Silicon and is unstable (random installer crashes, grey screens).

## 1. Create and provision the VM

1. Create the VM shell with **`infra/create_vm.sh`** (on the Mac host):
   ```bash
   ./infra/create_vm.sh   # arm64, EFI, 4 vCPU / 8 GB / 60 GB, ~/Downloads/ubuntu-24.04-live-server-arm64.iso attached
   # flags: --name --iso --cpus --ram --disk --recreate
   ```
   This drives `VBoxManage` directly, because VirtualBox 7.2's **"New" GUI wizard tends to
   crash on Apple Silicon** at the disk step. (If you prefer the GUI and it works for you: New
   → *Linux* / **Ubuntu (ARM 64-bit)**, 4 vCPUs, 8 GB RAM, 60 GB dynamic disk, attach the arm64
   ISO.) Then start it (`VBoxManage startvm eggcc-artifact`) and run the Ubuntu 24.04 installer:
   - username **`artifact`**, password **`artifact`**;
   - **give the root filesystem the whole disk.** Ubuntu's guided LVM leaves ~half the volume
     group unallocated by default, which is too small for the LLVM build + Gurobi + a full run.
     Edit `ubuntu-lv` to use all free space during install, or afterwards:
     `sudo lvextend -l +100%FREE /dev/ubuntu-vg/ubuntu-lv && sudo resize2fs /dev/ubuntu-vg/ubuntu-lv`.

   Never delete a VM by removing its folder in Finder — use VirtualBox's *Remove → Delete all
   files*, or `create_vm.sh --recreate`; a manual folder-delete leaves VirtualBox's config
   pointing at missing files.
2. Boot, log in, and provision:
   ```bash
   sudo apt-get update && sudo apt-get install -y git
   git clone https://github.com/egraphs-good/eggcc ~/eggcc
   bash ~/eggcc/infra/provision.sh --ref oflatt-gurobi-optional
   ```
   `provision.sh` installs a minimal GNOME desktop, all dependencies (LLVM 18, CBC, and the
   Gurobi *solver* — unlicensed), and builds eggcc. It does **not** generate any figures — you
   do that in step 2. Reboot into the desktop when it finishes. Login: `artifact` / `artifact`.

## 2. Generate the reference figures inside the VM

The shipped VM ships two reference sets for reviewers to compare against: `~/reference/` (the
default **CBC** run) and `~/reference/gurobi/` (the paper's **Gurobi** run). Generate both by
hand, `full` scale (~3–4 h each):

```bash
# CBC reference (no license needed):
eggcc/artifact/reproduce.sh full --out-dir ~/reference

# Gurobi reference: install your academic license first. Download gurobi.lic from Gurobi's Web
# License Manager on the host, scp it into the VM, and install it (grbgetkey also works for
# older Named-User licenses, but WLS/academic licenses need the file):
#   scp -P 2222 ~/Downloads/gurobi.lic artifact@localhost:~/   # on the HOST
eggcc/infra/setup_gurobi.sh ~/gurobi.lic   # verifies Gurobi is licensed
mkdir -p ~/reference/gurobi
eggcc/artifact/reproduce.sh full --out-dir ~/reference/gurobi   # tiger-vs-Gurobi-vs-CBC
```

So the shipped VM has `~/reference/` (CBC, matches the default run) and `~/reference/gurobi/`
(the paper's Gurobi comparison, plus a few Gurobi-only figures). Reviewers compare their own
run against whichever matches how they ran `reproduce.sh`.

> **Remove your license before exporting.** Your `~/gurobi.lic` is a personal academic
> credential — it must not ship in the OVA. Delete it before step 3:
> `rm -f ~/gurobi.lic`. (The Gurobi *solver* install is fine to ship; only the license is
> personal.)

## 3. Export and package for submission

First, **in the VM**, remove your personal Gurobi license *and* the license key from shell
history (you typed it into `grbgetkey`), so neither ships in the OVA, then shut the VM down:

```bash
rm -f ~/gurobi.lic                                   # the installed license
find ~ -name 'gurobi.lic' -delete 2>/dev/null        # any stray copies
unset HISTFILE; history -c; rm -f ~/.bash_history    # drop the license key from history
sudo poweroff
```

(The Gurobi *solver* install is fine to ship — only the license and the key are personal.)

Then, on the host, export the appliance:

```bash
# on the Mac host:
# detach the install ISO first, so it isn't bundled into the OVA and can't boot a reviewer
# back into the installer (do this with the VM powered off):
VBoxManage storageattach eggcc-artifact --storagectl SATA --port 1 --device 0 --type dvddrive --medium emptydrive
VBoxManage export eggcc-artifact -o eggcc-artifact.ova
```

Upload **two files** to a single Zenodo record — `eggcc-artifact.ova` and
`~/eggcc/artifact/README.md` — then submit the DOI to the AE HotCRP. Don't wrap them in a
zip/tarball: Zenodo records hold multiple files, and an OVA is already internally compressed
(the exported disk is a stream-optimized, gzipped VMDK), so an outer archive would shrink it by
only ~1–3% while forcing reviewers to unpack a multi-GB download. Uploading `README.md`
separately also lets Zenodo render it on the record page, so reviewers can read §0 (install
VirtualBox, import the OVA, log in) *before* downloading anything. That same file is waiting at
`~/README.md` inside the VM, where its reproduction guide takes over.

Because the OVA is arm64, an x86-only reviewer can't import it. The SPLASH/OOPSLA AE guidance
supports ARM VMs (VirtualBox 7.2), so an arm64 appliance is acceptable to submit.
