# Install

## Ubuntu 22.04 / 24.04

Download the package matching the destination Ubuntu version and architecture
from the [latest release](https://github.com/pregene/autobricks-worm/releases/latest),
along with its `SHA256SUMS-VERSION.txt` manifest. The release contains Ubuntu
22.04/24.04 packages for both `amd64` and `arm64`.

Verify downloaded packages and install with APT so dependencies are resolved.
Replace `<version>`, `<ubuntu-version>`, and `<arch>` with the downloaded values:

```sh
sha256sum --ignore-missing -c SHA256SUMS-<version>.txt
sudo apt install ./autobricks-worm-<version>-ubuntu-<ubuntu-version>-<arch>.deb
```

The package is named `autobricks-worm`; the executable, service, debconf settings,
and configuration path retain their `ab-worm` names. The package replaces the
older `ab-worm` package and declares the conflict so both cannot own the same
files. Use APT to resolve the replacement; do not purge the old configuration
first. Existing configured source and mount paths are reused by the installer.

For source builds and the full Docker matrix, see [PACKAGE.md](PACKAGE.md).
Generated packages are stored in the Git-ignored `build/` directory.

The installer asks for the retention period in days. It then creates:

```text
/worm-storage
/mnt/worm-storage
```

The systemd service runs as root and mounts:

```sh
/usr/bin/ab-worm mount /worm-storage /mnt/worm-storage --retain DAYS --allow-other
```

`/worm-storage` is root-only backing storage. Users write through `/mnt/worm-storage`.

To use different paths, edit `/etc/default/ab-worm`:

```sh
AB_WORM_SOURCE_PATH=/data/worm-storage
AB_WORM_MOUNT_PATH=/mnt/worm-storage
```

Then restart:

```sh
sudo systemctl restart ab-worm.service
```

Check service status:

```sh
systemctl status ab-worm.service
```
