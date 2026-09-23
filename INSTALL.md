# Install

## Ubuntu 22.04 / 24.04

Build the Debian package:

```sh
scripts/package_deb.sh
```

Build Ubuntu packages for `amd64` and `arm64` with Docker:

```sh
scripts/package_deb_docker.sh
AB_WORM_PACKAGE_IMAGE=ubuntu:24.04 AB_WORM_PACKAGE_OS_VERSION=24.04 scripts/package_deb_docker.sh
```

Generated packages are written to `build/`. The directory is ignored by Git
because release artifacts should be attached through the release process, not
committed to the source repository.

Install the package that matches the OS version and architecture:

```sh
sudo dpkg -i build/ab-worm-0.1.6-ubuntu-22.04-amd64.deb
```

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
