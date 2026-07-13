# syntax=docker/dockerfile:1

# =============================================================================
# eggcc artifact image (OOPSLA/SPLASH artifact evaluation).
#
# A self-contained Ubuntu 22.04 image that installs every dependency, clones and
# builds eggcc, and reproduces the paper's figures. It installs the same
# dependencies as the project's install_ubuntu.sh and nightly (LLVM 18, CBC, Rust,
# Python); it is headless -- reviewers view the generated PDFs on the host through a
# bind-mounted output directory.
#
# Build + run instructions live in infra/build_docker.sh and infra/BUILDING.md.
# Quick version:
#   docker build -t eggcc-artifact infra/                     # single-platform
#   mkdir -p out
#   docker run --rm -it -v "$PWD/out:/out" eggcc-artifact ./reproduce.sh smoke
#   # figures land in ./out on the host; use `full` for the ~3-4 h suite.
#
# Pin EGGCC_REF to a commit SHA for an exactly-reproducible image, e.g.:
#   docker build --build-arg EGGCC_REF=<sha> -t eggcc-artifact infra/
# =============================================================================

FROM ubuntu:22.04

ARG EGGCC_REPO=https://github.com/egraphs-good/eggcc
ARG EGGCC_REF=oflatt-gurobi-optional
# The runtime is built with a pinned nightly (LLVM 18.1.4); see runtime/install.sh.
ARG RUST_NIGHTLY=nightly-2024-05-01

ENV DEBIAN_FRONTEND=noninteractive \
    LLVM_SYS_180_PREFIX=/usr/lib/llvm-18/ \
    RUSTUP_HOME=/root/.rustup \
    CARGO_HOME=/root/.cargo \
    PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

# --- Base tooling: git/curl, graphviz (`dot`) for CFGs, python for graphs -----
RUN apt-get update \
 && apt-get install -y \
      ca-certificates git curl gnupg build-essential make \
      graphviz \
      python3 python3-venv python3-pip \
 && rm -rf /var/lib/apt/lists/*

# --- LLVM 18 (eggcc needs clang-18 / llvm-18 / llvm-config-18) -----------------
# Mirrors install_ubuntu.sh / provision.sh, using a modern signed keyring.
RUN curl -fsSL https://apt.llvm.org/llvm-snapshot.gpg.key \
      | gpg --dearmor -o /usr/share/keyrings/llvm-snapshot.gpg \
 && echo "deb [signed-by=/usr/share/keyrings/llvm-snapshot.gpg] http://apt.llvm.org/jammy/ llvm-toolchain-jammy-18 main" \
      > /etc/apt/sources.list.d/llvm-18.list \
 && apt-get update \
 && apt-get install -y \
      clang-18 llvm-18 libllvm18 llvm-18-dev llvm-18-runtime \
      libpolly-18-dev libzstd-dev zlib1g-dev \
 && rm -rf /var/lib/apt/lists/*

# --- CBC (the default free ILP solver; Gurobi is optional, installed at runtime)
# coinor-libcbc-dev: linked by the coin_cbc crate.
# coinor-cbc:        the `cbc` CLI the Statewalk DP extractor (`tiger`) shells out to.
RUN apt-get update \
 && apt-get install -y coinor-libcbc-dev coinor-cbc \
 && rm -rf /var/lib/apt/lists/*

# --- Rust: default 1.88.0 (per rust-toolchain) + the runtime's pinned nightly --
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --no-modify-path --default-toolchain 1.88.0 --profile minimal \
 && rustup toolchain install "$RUST_NIGHTLY" --profile minimal

# --- Clone + build eggcc + the bril LLVM runtime -------------------------------
WORKDIR /root
RUN git clone "$EGGCC_REPO" eggcc \
 && git -C eggcc fetch origin "$EGGCC_REF" \
 && git -C eggcc checkout "$EGGCC_REF"
WORKDIR /root/eggcc
RUN make runtime \
 && cargo build --release

# --- tokei (the nightly's line-count table) + pinned python deps in the venv ---
# reproduce.sh looks for the interpreter at $HOME/.eggcc-venv and cargo at $HOME/.cargo.
RUN cargo install tokei --version 13.0.0 --locked
RUN python3 -m venv /root/.eggcc-venv \
 && /root/.eggcc-venv/bin/pip install --no-cache-dir --upgrade pip \
 && /root/.eggcc-venv/bin/pip install --no-cache-dir -r infra/requirements.txt

# --- reviewer entry point: ./reproduce.sh, figures -> /out ---------------------
# Bind-mount a host directory to /out (`-v "$PWD/out:/out"`) to collect the PDFs.
RUN printf '#!/usr/bin/env bash\nexec /root/eggcc/artifact/reproduce.sh --out-dir /out "$@"\n' \
      > /root/reproduce.sh \
 && chmod +x /root/reproduce.sh

VOLUME /out
WORKDIR /root
CMD ["bash"]
