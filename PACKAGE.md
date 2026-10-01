# Building installation packages

Autobricks WORM provides the `autobricks-worm` Debian package containing the
`ab-worm` executable, systemd service, interactive retention configuration,
documentation, and license notices. Run commands from the repository root.

## Build on the current host

Install Bash, Python 3, a Rust toolchain supporting the locked dependencies
(the release uses Rust 1.98.1), a C compiler, `pkg-config`, `binutils`, and
`dpkg-dev`. Installation requires FUSE 3 and systemd.

```sh
scripts/package_deb.sh
scripts/verify_package.sh
```

The builder stages the executable in `bin/` and packages in `build/`. Filenames
use the actual build distribution from `/etc/os-release` and target architecture:

```text
autobricks-worm-VERSION-OS-OS_VERSION-ARCH.deb
```

Each packaging invocation increments the patch component of `VERSION` before
the build attempt, including attempts that fail. A build lock serializes
version allocation. The package and executable use the same allocated version.
Do not rename packages to represent a different distribution or architecture.

## Ubuntu release matrix

Use Docker Engine on an amd64 host with Docker access and network access for
build dependencies:

```sh
scripts/package_deb_docker.sh
```

| Ubuntu | Architectures | Package |
| --- | --- | --- |
| 22.04 | amd64, arm64 | autobricks-worm |
| 24.04 | amd64, arm64 | autobricks-worm |

All four packages share one allocated version. Builds run in temporary native
amd64 containers using Rust 1.98.1 and locked Cargo dependencies. ARM64 uses
the GNU AArch64 cross-compiler and target distribution libraries; no QEMU or
binfmt registration is needed. The source checkout is mounted read-only, and
containers and temporary staging directories are removed at completion.

The builder checks package identity, version, architecture, packaged binary
contents, installer shell syntax, license payloads, and target ELF libraries.
amd64 binaries also run version/help checks. ARM64 execution and package
installation must be checked on matching target hosts; the matrix does not
install packages on the build host.

Only after every target passes are the four packages copied into `build/`
together with `SHA256SUMS-VERSION.txt`:

```sh
cd build
sha256sum -c SHA256SUMS-<version>.txt
```

## Release contents

Publish the four verified packages and checksum manifest with their matching
source version. Verify uploaded asset sizes and SHA-256 hashes before removing
previous release posts and their attached files. Keep repository tags. The
release list contains the current release rather than accumulating old posts.

[Install packages](INSTALL.md) · [Filesystem features](README.md)
