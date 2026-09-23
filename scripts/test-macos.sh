#!/bin/sh
set -eu

if [ "$(uname -s)" != Darwin ]; then
    echo "This test command requires macOS." >&2
    exit 1
fi

case "${1:-}" in
    ""|--mount) ;;
    *) echo "Usage: scripts/test-macos.sh [--mount]" >&2; exit 2 ;;
esac
if [ "$#" -gt 1 ]; then
    echo "Usage: scripts/test-macos.sh [--mount]" >&2
    exit 2
fi

export PKG_CONFIG_PATH="${PKG_CONFIG_PATH:+$PKG_CONFIG_PATH:}/usr/local/lib/pkgconfig"
if ! command -v pkg-config >/dev/null 2>&1 || ! pkg-config --exists fuse; then
    echo "Install macFUSE and pkg-config, then rerun this command:" >&2
    echo "  brew install --cask macfuse" >&2
    echo "  brew install pkg-config" >&2
    exit 1
fi

cd "$(dirname "$0")/.."
if [ "${1:-}" = --mount ]; then
    if [ ! -d /Library/Filesystems/macfuse.fs ]; then
        echo "Install the macFUSE runtime before running the mount test." >&2
        exit 1
    fi
    exec cargo test --locked --features macos-fuse --test fuse_namespace -- --ignored --nocapture
fi

cargo test --locked --features macos-fuse
cargo clippy --locked --features macos-fuse --all-targets -- -D warnings
