# macOS testing

## Setup

Install the macFUSE runtime and pkg-config:

```sh
brew install --cask macfuse
brew install pkg-config
```

The macFUSE package installer uses administrator authentication. Its installer and macOS System Settings handle runtime authorization. The fuser 0.16 adapter uses macFUSE's device-descriptor interface and kernel backend. Follow the [official runtime setup instructions](https://github.com/macfuse/macfuse/wiki/Getting-Started) for that backend.

## Build and policy tests

```sh
./scripts/test-macos.sh
```

This command enables the `macos-fuse` Cargo feature, locates the installed `fuse.pc`, compiles the macOS adapter, runs policy tests, and runs Clippy.

To build a versioned executable with the adapter enabled:

```sh
PKG_CONFIG_PATH=/usr/local/lib/pkgconfig ./build.sh --release --features macos-fuse
```

## Native mount test

With the macFUSE kernel backend available:

```sh
./scripts/test-macos.sh --mount
```

The test mounts an isolated fixture, exercises ordinary file, directory, link, and FIFO creation, and verifies that `.meta` creation and rename requests return `EPERM` before reaching the backing filesystem. The fixture is unmounted and its mountpoint removed when the test finishes.

The macOS adapter additionally rejects exchange and volume-renaming callbacks and forwards extended timestamp queries.

## Backend interface

The adapter shares FUSE namespace checks with Linux, with macOS-specific callbacks in `src/macos/fuse.rs`.

macFUSE's FSKit backend uses a different transport. Its [MFMount developer documentation](https://github.com/macfuse/macfuse/wiki/Getting-Started-%28Developer%29-%E2%80%90-MFMount.framework) describes the channel API, while the selected fuser version mounts through `fuse_mount_compat25` and reads requests from a file descriptor.
