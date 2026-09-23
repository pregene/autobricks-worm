#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(tr -d '[:space:]' < VERSION)"
ARCH="${AB_WORM_DEB_ARCH:-$(dpkg --print-architecture)}"
OS_NAME="${AB_WORM_PACKAGE_OS:-ubuntu}"
OS_VERSION="${AB_WORM_PACKAGE_OS_VERSION:-22.04}"
BUILD_ROOT="$ROOT/build"
PACKAGE_ROOT="$BUILD_ROOT/ab-worm_${VERSION}_${ARCH}"
OUTPUT="$BUILD_ROOT/ab-worm-${VERSION}-${OS_NAME}-${OS_VERSION}-${ARCH}.deb"
TARGET_ROOT="${CARGO_TARGET_DIR:-$ROOT/target}"
RUST_TARGET="${AB_WORM_RUST_TARGET:-}"

if [[ -n "$RUST_TARGET" ]]; then
  cargo build --release --locked --target "$RUST_TARGET"
  BINARY="$TARGET_ROOT/$RUST_TARGET/release/ab-worm"
else
  cargo build --release --locked
  BINARY="$TARGET_ROOT/release/ab-worm"
fi

rm -rf "$PACKAGE_ROOT"
install -d "$PACKAGE_ROOT/DEBIAN"
install -d "$PACKAGE_ROOT/usr/bin"
install -d "$PACKAGE_ROOT/lib/systemd/system"
install -d "$PACKAGE_ROOT/usr/share/doc/ab-worm"

install -m 0755 "$BINARY" "$PACKAGE_ROOT/usr/bin/ab-worm"
install -m 0644 packaging/linux/ab-worm.service "$PACKAGE_ROOT/lib/systemd/system/ab-worm.service"
install -m 0644 README.md "$PACKAGE_ROOT/usr/share/doc/ab-worm/README.md"
install -m 0644 LICENSE "$PACKAGE_ROOT/usr/share/doc/ab-worm/copyright"

cat >"$PACKAGE_ROOT/DEBIAN/control" <<EOF
Package: ab-worm
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Depends: fuse3, debconf, systemd
Maintainer: Autobricks, Co. <development@autobricks.invalid>
Description: Autobricks WORM Filesystem
 Appendable WORM filesystem service mounted from /worm-storage to /mnt/worm-storage.
EOF

install -m 0755 packaging/linux/debian-config "$PACKAGE_ROOT/DEBIAN/config"
install -m 0644 packaging/linux/debian-templates "$PACKAGE_ROOT/DEBIAN/templates"
install -m 0755 packaging/linux/debian-postinst "$PACKAGE_ROOT/DEBIAN/postinst"
install -m 0755 packaging/linux/debian-prerm "$PACKAGE_ROOT/DEBIAN/prerm"
install -m 0755 packaging/linux/debian-postrm "$PACKAGE_ROOT/DEBIAN/postrm"

dpkg-deb --root-owner-group --build "$PACKAGE_ROOT" "$OUTPUT"
echo "$OUTPUT"
