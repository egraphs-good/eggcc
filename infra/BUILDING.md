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

1. Create an Ubuntu 22.04 Desktop VM in the VirtualBox GUI (4 vCPUs, 8 GB RAM, 60 GB disk;
   user `eggcc`).
2. Inside the VM:
   ```bash
   sudo apt-get update && sudo apt-get install -y git
   git clone https://github.com/egraphs-good/eggcc ~/eggcc
   bash ~/eggcc/infra/provision.sh --ref oflatt-gurobi-optional
   ```
3. `cd ~/eggcc && artifact/reproduce.sh smoke` to confirm, then do steps 2–3 above.

## Docker image (alternative to the VM)

The SPLASH packaging guidance accepts a Docker image as well as a VM, and it avoids
VirtualBox entirely — useful because VirtualBox 7.2 on Apple-Silicon Macs can only run ARM
guests (so an amd64 VM can't be built there). `infra/Dockerfile` builds the same artifact as
`provision.sh`, minus the VM-only GNOME desktop; reviewers view the generated PDFs on the
host through a bind-mounted directory.

### How the image is built

`infra/Dockerfile` starts from `ubuntu:22.04` and runs the same steps as the tested
`provision.sh`: LLVM 18 (from apt.llvm.org), CBC, Rust `1.88.0` (per `rust-toolchain`) plus
the `nightly-2024-05-01` the runtime needs, then `make runtime && cargo build --release`, the
pinned Python deps in `/root/.eggcc-venv`, and a `/root/reproduce.sh` wrapper that writes
figures to `/out`. It clones eggcc at `--build-arg EGGCC_REF` (default the artifact branch);
pin a commit SHA for an exactly-reproducible image.

```bash
cd eggcc

# Single-platform image (amd64 = the paper's architecture), loaded locally:
infra/build_docker.sh
#   ≡ docker build --build-arg EGGCC_REF=oflatt-gurobi-optional -t eggcc-artifact infra/

# Multi-platform image, per the guidance, pushed to a registry:
docker buildx create --use --name eggcc-builder            # once
infra/build_docker.sh --platforms linux/amd64,linux/arm64 \
  --tag ghcr.io/<you>/eggcc-artifact:oopsla26 --push
#   ≡ docker buildx build --platform linux/amd64,linux/arm64 \
#       --build-arg EGGCC_REF=... -t <tag> --push infra/
```

Multi-platform notes: Docker can't load a multi-arch manifest locally, so `--platforms`
requires `--push`. Building a non-native arch uses QEMU emulation and is slow for a
Rust+LLVM build — for the release, prefer native runners (a CI matrix building amd64 and
arm64 separately). amd64 is the reference; arm64 is a convenience for Apple-Silicon reviewers
and depends on apt.llvm.org serving arm64 for jammy.

### Run / smoke-test

```bash
mkdir -p out
docker run --rm -it -v "$PWD/out:/out" eggcc-artifact ./reproduce.sh smoke   # ~5-10 min
docker run --rm -it -v "$PWD/out:/out" eggcc-artifact ./reproduce.sh full    # ~3-4 h
# figures appear in ./out on the host.
```

### Ship it

Submit either a registry reference or an offline tarball with a DOI (Zenodo):

```bash
docker save eggcc-artifact | gzip > eggcc-artifact-image.tar.gz   # reviewers: docker load < ...
```
