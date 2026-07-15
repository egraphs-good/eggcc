# Building and shipping the eggcc artifact VM

Author-facing guide for producing `eggcc-artifact.ova` on a macOS host. Reviewers don't need
this — they import and run the VM following the host-side setup guide
(`artifact/HOST_README.md`, shipped next to the OVA), then the in-VM `README.md`.

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
overwrite with the Gurobi run in step 2. `build_vm.sh` downloads the Ubuntu 22.04 **Server**
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

Provisioning already pre-filled `~/reference/` with the default **CBC** `full` run — that is
what most reviewers reproduce — and installed the **Gurobi solver** (unlicensed). Add your own
license and produce the paper's **Gurobi** run under `~/reference/gurobi/`, exactly as a
licensed reviewer would (see the README's "Optional — Gurobi" for the `grbgetkey` flow):

```bash
# in the VM — activate your academic license (writes ~/gurobi.lic), then:
grbgetkey <YOUR-KEY>                    # or drop a gurobi.lic in the home folder
eggcc/infra/setup_gurobi.sh            # verifies Gurobi is licensed
mkdir -p ~/reference/gurobi
eggcc/artifact/reproduce.sh full --out-dir ~/reference/gurobi   # ~3–4 h; tiger-vs-Gurobi-vs-CBC
```

So the shipped VM has `~/reference/` (CBC, matches the default run) and `~/reference/gurobi/`
(the paper's Gurobi comparison, plus a few Gurobi-only figures). Reviewers compare their own
run against whichever matches how they ran `reproduce.sh`.

> **Remove your license before exporting.** Your `~/gurobi.lic` is a personal academic
> credential — it must not ship in the OVA. Delete it before step 3:
> `rm -f ~/gurobi.lic`. (The Gurobi *solver* install is fine to ship; only the license is
> personal.)

## 3. Export and package for submission

First, **in the VM**, remove your personal Gurobi license so it isn't distributed, then shut
the VM down:

```bash
rm -f ~/gurobi.lic      # in the VM — do NOT ship your academic license
sudo poweroff
```

Then, on the host, export the appliance and bundle it with the host-side setup guide (as
`README.md`) into a single zip for Zenodo:

```bash
# on the Mac host (adjust the path to your eggcc checkout):
VBoxManage export eggcc-artifact -o eggcc-artifact.ova

mkdir -p eggcc-artifact
cp eggcc-artifact.ova              eggcc-artifact/
cp ~/eggcc/artifact/HOST_README.md eggcc-artifact/README.md
zip -r eggcc-artifact.zip eggcc-artifact
```

Upload `eggcc-artifact.zip` to Zenodo for a DOI, then submit the DOI to the AE HotCRP. The
zip's `README.md` (the host-side setup guide) tells reviewers how to install VirtualBox,
import the OVA, and log in, then points them at the in-VM guide. (Reviewers can't read the
in-VM `README.md` until they've booted the VM, so this host-side one is required.)

## Manual fallback

`build_vm.sh` is the only part of the artifact not tested by its author (the build
environment had no VirtualBox). If a `VBoxManage` step fails on your host, build the VM by
hand instead:

1. Create an Ubuntu 22.04 Desktop VM in the VirtualBox GUI (4 vCPUs, 8 GB RAM, 60 GB disk;
   user `eggcc`).
2. Inside the VM:
   ```bash
   sudo apt-get update && sudo apt-get install -y git
   git clone https://github.com/egraphs-good/eggcc ~/eggcc
   bash ~/eggcc/infra/provision.sh --ref oflatt-gurobi-optional
   ```
3. `cd ~/eggcc && artifact/reproduce.sh smoke` to confirm, then do steps 2–3 above.
