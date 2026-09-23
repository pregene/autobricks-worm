#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

IMAGE="${AB_WORM_PACKAGE_IMAGE:-ubuntu:22.04}"
PACKAGE_OS="${AB_WORM_PACKAGE_OS:-ubuntu}"
PACKAGE_OS_VERSION="${AB_WORM_PACKAGE_OS_VERSION:-22.04}"
ARCHES=("$@")

if [[ "$#" -eq 0 ]]; then
  ARCHES=(amd64 arm64)
fi

HOST_UID="$(id -u)"
HOST_GID="$(id -g)"

for arch in "${ARCHES[@]}"; do
  case "$arch" in
    amd64)
      apt_extra=""
      rust_target=""
      env_args=()
      ;;
    arm64)
      apt_extra="gcc-aarch64-linux-gnu libc6-dev-arm64-cross"
      rust_target="aarch64-unknown-linux-gnu"
      env_args=(-e CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc)
      ;;
    *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
  esac

  echo "== Building $arch package on $IMAGE =="
  docker run --rm \
    --platform linux/amd64 \
    -e DEBIAN_FRONTEND=noninteractive \
    -e AB_WORM_DEB_ARCH="$arch" \
    -e AB_WORM_PACKAGE_OS="$PACKAGE_OS" \
    -e AB_WORM_PACKAGE_OS_VERSION="$PACKAGE_OS_VERSION" \
    -e AB_WORM_RUST_TARGET="$rust_target" \
    -e CARGO_TARGET_DIR=/tmp/ab-worm-target \
    "${env_args[@]}" \
    -e AB_WORM_APT_EXTRA="$apt_extra" \
    -e HOST_UID="$HOST_UID" \
    -e HOST_GID="$HOST_GID" \
    -v "$ROOT:/work" \
    -w /work \
    "$IMAGE" \
    bash -lc '
      set -euo pipefail
      apt-get update
      apt-get install -y --no-install-recommends ca-certificates curl build-essential pkg-config dpkg-dev ${AB_WORM_APT_EXTRA}
      if ! command -v cargo >/dev/null 2>&1; then
        curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.85.0
        . "$HOME/.cargo/env"
      fi
      if [ -n "${AB_WORM_RUST_TARGET}" ]; then
        rustup target add "${AB_WORM_RUST_TARGET}"
      fi
      scripts/package_deb.sh
      chown -R "$HOST_UID:$HOST_GID" build
    '
done

echo
ls -1 build/ab-worm-*-*.deb
