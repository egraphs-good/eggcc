#!/usr/bin/env bash
#
# Build the eggcc artifact container image (OOPSLA/SPLASH artifact evaluation).
# The Docker counterpart to build_vm.sh; see infra/Dockerfile and infra/BUILDING.md.
#
#   ./build_docker.sh [--tag TAG] [--ref REF] [--platforms LIST] [--push]
#
# Examples:
#   # Local single-platform image (matches the paper's x86_64 numbers), loaded into docker:
#   ./build_docker.sh
#
#   # Multi-platform image pushed to a registry (per the SPLASH packaging guidance):
#   ./build_docker.sh --platforms linux/amd64,linux/arm64 \
#                     --tag ghcr.io/<you>/eggcc-artifact:oopsla26 --push
#
# Notes:
#  - Single-platform uses plain `docker build` and loads the image into the local engine.
#  - Multi-platform uses `docker buildx` and REQUIRES --push to a registry: Docker cannot
#    load a multi-arch manifest into the local engine. It also needs a buildx builder once:
#        docker buildx create --use --name eggcc-builder
#  - Building a non-native architecture uses QEMU emulation and is slow (eggcc is a large
#    Rust+LLVM build). For a real release, prefer native runners -- e.g. a CI matrix that
#    builds linux/amd64 and linux/arm64 separately and stitches the manifest.
#  - amd64 is the reference architecture (the paper's numbers are x86_64); arm64 is a
#    convenience so Apple-Silicon reviewers can run natively.
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TAG="eggcc-artifact"
REF="oflatt-gurobi-optional"
PLATFORMS=""
PUSH=0

while [ $# -gt 0 ]; do
  case "$1" in
    --tag)       TAG="${2:?}"; shift 2 ;;
    --ref)       REF="${2:?}"; shift 2 ;;
    --platforms) PLATFORMS="${2:?}"; shift 2 ;;
    --push)      PUSH=1; shift ;;
    -h|--help)   sed -n '2,30p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

command -v docker >/dev/null || { echo "ERROR: docker not found. Install Docker Desktop / Engine."; exit 1; }

if [ -z "$PLATFORMS" ]; then
  echo "Building single-platform image '$TAG' (eggcc ref: $REF)..."
  docker build --build-arg EGGCC_REF="$REF" -t "$TAG" "$DIR"
  echo ""
  echo "Done. Run it with:"
  echo "  mkdir -p out && docker run --rm -it -v \"\$PWD/out:/out\" $TAG ./reproduce.sh smoke"
else
  [ "$PUSH" -eq 1 ] || { echo "ERROR: --platforms requires --push (a multi-arch image cannot be loaded locally)."; exit 1; }
  echo "Building multi-platform image '$TAG' [$PLATFORMS] and pushing (eggcc ref: $REF)..."
  docker buildx build --platform "$PLATFORMS" --build-arg EGGCC_REF="$REF" -t "$TAG" --push "$DIR"
  echo ""
  echo "Done. Pushed $TAG for: $PLATFORMS"
fi
