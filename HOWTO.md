# HOWTO

## Use The Mounted Filesystem

After installation, use:

```sh
/mnt/worm-storage
```

Any logged-in user can create directories and append to files through the mount.
Committed bytes cannot be overwritten, paths cannot be renamed, and retained files
cannot be deleted until their retention period expires.

Each data file has a read-only metadata file:

```text
example.log
example.log.meta
```

## Change Retention For New Files

Edit:

```sh
sudo editor /etc/default/ab-worm
```

Set:

```sh
AB_WORM_RETAIN_DAYS=365
```

The backing and mount paths are also configured there:

```sh
AB_WORM_SOURCE_PATH=/worm-storage
AB_WORM_MOUNT_PATH=/mnt/worm-storage
```

Restart:

```sh
sudo systemctl restart ab-worm.service
```

Existing files keep their original retention deadlines.
