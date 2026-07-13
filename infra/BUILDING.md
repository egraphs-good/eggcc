# Building and shipping the eggcc artifact VM

Author-facing guide for producing `eggcc-artifact.ova` on a macOS host. Reviewers don't need
this — they just import and run the VM (see `README.md`).

## 0. Prerequisites

- **Push the branch to GitHub first.** `build_vm.sh` provisions the VM by cloning eggcc from
  GitHub at `--ref` (default `oflatt-gurobi-optional`), so the `artifact/` scripts must be on
  that branch *on GitHub* before you build. (Once the paper's code lands on `main`, pass
  `--ref main` or a tagged commit.)
- **VirtualBox 7.2+** on the Mac: `brew install --cask virtualbox`.

## 1. Build the VM

```bash
git clone https://github.com/egraphs-good/eggcc      # or `git pull` an existing clone
cd eggcc && git checkout oflatt-gurobi-optional
cd infra
./build_vm.sh --pregenerate smoke
```

`--pregenerate smoke` keeps the build fast — it skips the ~3–4 h CBC full run you'd just
overwrite with the Gurobi run in step 2. `build_vm.sh` downloads the Ubuntu 24.04 **Server**
ISO (~2 GB) and unattended-installs it (VirtualBox drives the server installer reliably; the
desktop ISO's newer installer fails at VBox's "prepare" step). It then provisions over SSH:
`provision.sh` adds a minimal GNOME desktop so the VM is graphical, installs the deps, builds
eggcc, and runs the smoke check. Expect ~1 hour. Login: `eggcc` / `eggcc`.

Flags: `--cpus`, `--ram`, `--disk`, `--ssh-port`, `--ref`, `--pregenerate full|smoke|none`,
`--iso-url`, `--workdir`.

### Re-running from a clean state

`build_vm.sh` reuses an existing VM named `eggcc-artifact`, so a failed or partial run must be
torn down first or the next run inherits its broken state:

```bash
VBoxManage controlvm eggcc-artifact poweroff 2>/dev/null || true
VBoxManage unregistervm eggcc-artifact --delete 2>/dev/null || true
rm -f ~/eggcc-artifact-build/eggcc-artifact.vdi
```

The SSH keypair and any downloaded ISO in `~/eggcc-artifact-build/` are reused across runs, so
leave them. (Delete the whole `~/eggcc-artifact-build/` only if you also want to re-download
the ISO.)

## 2. Generate the shipped figures (with Gurobi) inside the VM

The reference figures are produced *in the VM* with a Gurobi license, exactly as a licensed
reviewer would:

```bash
# in the VM — drop your gurobi.lic in the home folder first:
eggcc/infra/setup_gurobi.sh ~/gurobi.lic
./reproduce.sh full          # ~3–4 h; the tiger-vs-Gurobi-vs-CBC figures land in ~/
```

## 3. Export the appliance

```bash
# on the Mac host:
VBoxManage export eggcc-artifact -o eggcc-artifact.ova
```

Then submit `eggcc-artifact.ova` (e.g. upload to Zenodo for a DOI, then the AE HotCRP).

## Manual fallback

`build_vm.sh` is the only part of the artifact not tested by its author (the build
environment had no VirtualBox). If a `VBoxManage` step fails on your host, build the VM by
hand instead:

1. Create an Ubuntu 24.04 Desktop VM in the VirtualBox GUI (4 vCPUs, 8 GB RAM, 40 GB disk;
   user `eggcc`).
2. Inside the VM:
   ```bash
   sudo apt-get update && sudo apt-get install -y git
   git clone https://github.com/egraphs-good/eggcc ~/eggcc
   bash ~/eggcc/infra/provision.sh --ref oflatt-gurobi-optional
   ```
3. `cd ~/eggcc && artifact/reproduce.sh smoke` to confirm, then do steps 2–3 above.
