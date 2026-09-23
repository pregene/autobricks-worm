# macOS testing

## Requirements

- macOS 15.4 or later for Apple FSKit.
- Apple developer tools with Swift and the macOS 15.4 or newer SDK.
- Rust/Cargo and Python 3.

Install Apple Command Line Tools if needed:

```sh
xcode-select --install
```

## Tests

```sh
./scripts/test-macos.sh
```

This runs Rust policy tests, Clippy, and native FSKit callback tests linked to the Rust policy library. The callback tests cover regular creation, reserved `.meta` names, invalid names, rename replacement, volume renaming, byte-preserving filename validation, and propagation of backend write failures.

## Build

```sh
./build.sh --release --locked
./target/release/ab-worm --version
```

The native build produces `ab-worm`, `libautobricks_worm.a`, and `libab_worm_fskit.dylib` with its Swift module in `target/release/`. The Swift adapter links to Apple's installed FSKit and Foundation frameworks. The deployment target defaults to the Swift toolchain target, with a minimum of macOS 15.4; an explicit `MACOSX_DEPLOYMENT_TARGET` must also be supported by the Rust toolchain and its standard library.

## Extension integration

`GuardedFileSystem` wraps an `FSUnaryFileSystem` backend and returns a `NamespaceGuard` for each successfully loaded volume. The backend provides volume operations, read/write operations, and open/close operations. Namespace checks run through the Rust C ABI before creation dispatch.

Apple's [FSKit extension guide](https://developer.apple.com/documentation/fskit/building-a-passthrough-file-system) describes the containing app, `UnaryFileSystemExtension` entry point, filesystem entitlement, and extension registration. Users enable a registered extension under System Settings > General > Login Items & Extensions > File System Extensions.
