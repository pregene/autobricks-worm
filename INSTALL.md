# Install

## Ubuntu

Build the Debian package:

```sh
scripts/package_deb.sh
```

Build Ubuntu 22.04 packages for `amd64` and `arm64` with Docker:

```sh
scripts/package_deb_docker.sh
```

Generated packages are written to `build/`. The directory is ignored by Git
because release artifacts should be attached through the release process, not
committed to the source repository.

Install it:

```sh
sudo dpkg -i build/ab-worm-*-ubuntu-22.04-amd64.deb
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
