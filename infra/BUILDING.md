# Building and publishing the eggcc artifact image

Author-facing guide for producing and publishing the `eggcc-artifact` **Docker image**.
Reviewers don't need this — they just load/pull and run the image (see `../artifact/README.md`).

## 0. Prerequisites

- **Push the branch to GitHub first.** `infra/Dockerfile` clones eggcc from GitHub at
  `--build-arg EGGCC_REF` (default `oflatt-gurobi-optional`), so the code must be on that
  branch *on GitHub* before you build. (Once the paper's code lands on `main`, build with
  `--build-arg EGGCC_REF=main` or a tagged commit / SHA.)
- **Docker with Buildx** — Docker Desktop, or Docker Engine plus the `buildx` plugin.

## 1. How the image is built

`infra/Dockerfile` starts from `ubuntu:22.04` and installs the same dependencies the project's
`install_ubuntu.sh` and nightly use: LLVM 18 (from apt.llvm.org), CBC, Rust `1.88.0` (per
`rust-toolchain`) plus the `nightly-2024-05-01` the runtime needs, then
`make runtime && cargo build --release`, the pinned Python deps in `/root/.eggcc-venv`, and a
`/root/reproduce.sh` wrapper that writes figures to `/out`. It clones eggcc at
`--build-arg EGGCC_REF`; pin a commit SHA for an exactly-reproducible image.

## 2. Build

```bash
cd eggcc

# Single-platform image (amd64 = the paper's architecture), loaded locally:
infra/build_docker.sh
#   ≡ docker build --build-arg EGGCC_REF=oflatt-gurobi-optional -t eggcc-artifact infra/

# Multi-platform image, per the SPLASH packaging guidance, pushed to a registry:
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

## 3. Smoke-test the image

```bash
mkdir -p out
docker run --rm -it -v "$PWD/out:/out" eggcc-artifact ./reproduce.sh smoke   # ~5-10 min
```

## 4. Reference figures (optional, with Gurobi)

The paper's figures use Gurobi, exactly as a licensed reviewer would. Mount your license and
run the full suite; the figures are produced at run time (not baked into the image):

```bash
docker run --rm -it -v "$PWD/out:/out" -v "$PWD/gurobi.lic:/root/gurobi.lic" \
  eggcc-artifact bash -c "infra/setup_gurobi.sh /root/gurobi.lic && ./reproduce.sh full"
# ~3-4 h; the tiger-vs-Gurobi-vs-CBC figures land in ./out
```

## 5. Publish

Submit either a registry reference or an offline tarball with a DOI (Zenodo), then the AE
HotCRP:

```bash
docker save eggcc-artifact | gzip > eggcc-artifact-image.tar.gz   # reviewers: docker load < ...
```
