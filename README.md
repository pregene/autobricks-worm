# Autobricks WORM Filesystem — Appendable WORM

A Rust project by **Autobricks, Co.** for **Appendable WORM (Write Once, Read Many)**: committed content stays immutable while new content can be appended to the same file.

The implemented policy core advances the LOCK boundary for each accepted append and keeps retention fixed from file creation. The command-line executable is **`ab-worm`**.

```text
[Record A]                       → LOCK after A
[Record A][Record B]             → LOCK after B
[Record A][Record B][Record C]    → LOCK after C

Each append preserves all previously committed records.
```

## Build

Requirements: Rust/Cargo and Python 3.

macOS filesystem mount testing additionally requires macOS 26+, Apple Developer Program membership (or an enrolled team), and a signed FSKit extension. See [macOS development and testing](docs/MACOS_TESTING.md) for account and signing setup.

Linux and macOS:

```sh
./build.sh --release
./target/release/ab-worm --version
```

For Ubuntu service installation with `/worm-storage` mounted at `/mnt/worm-storage`, see [INSTALL.md](INSTALL.md) and [HOWTO.md](HOWTO.md).

Windows (PowerShell):

```powershell
.\build.ps1 --release
.\target\release\ab-worm.exe --version
```

## CLI

```sh
ab-worm
ab-worm --help
ab-worm --version
```

Every invocation prints the product banner with the embedded version:

```text
Autobricks WORM Filesystem 0.1.1 (C) 2026 Autobricks, Co.
```

Running with no arguments or `--help` displays usage. `--version` displays the banner. The short options are `-h` and `-V`. Invalid arguments produce an error and a nonzero exit status.

## Backing storage test

Use a private, initially empty directory for backing data. The local test directory is `worm-storage/`.

```sh
./target/release/ab-worm storage ./worm-storage create example.log 3600
printf 'first record\n' | ./target/release/ab-worm storage ./worm-storage append example.log
printf 'second record\n' | ./target/release/ab-worm storage ./worm-storage append example.log
./target/release/ab-worm storage ./worm-storage verify example.log
./target/release/ab-worm storage ./worm-storage meta example.log
```

Each accepted append persists data, the standard SHA-256 checksum, incremental hash state, and LOCK. Retention starts at creation. Storage commands also support `read`, `mkdir`, and deletion after retention expires. The storage handle holds an exclusive process lock, and reopening recovers interrupted transactions. On Unix, the backing directory is restricted to its owner.

## Appendable WORM policy core

`FilePolicy` provides in-memory policy calculations using three fields:

| Field | Meaning |
| --- | --- |
| `created_at` | File creation time in UTC Unix seconds |
| `retain_until` | Fixed retention deadline in UTC Unix seconds |
| `lock_offset` | End of the committed data region, in bytes |

`FilePolicy::new(created_at, retention_seconds)` calculates:

```text
retain_until = created_at + retention_seconds
```

`after_append(offset, length)` returns an updated policy when the supplied offset equals the current LOCK boundary. It advances LOCK by the appended length and preserves the creation time and retention deadline. Other offsets return `NotAtEnd`.

`check_delete(now)` returns `RetentionActive` before the deadline and succeeds at or after it. Append validation continues to use the LOCK boundary after retention expires. Timestamp and offset arithmetic return `Overflow` when their values exceed the supported range.

`EntryPolicy` represents a file, directory, or metadata entry. Its `check_rename()` policy rejects name changes and moves with `ImmutablePath`, including replacement and exchange operations. A created entry retains its name and parent directory, including after file retention expires.

## Metadata policy

The metadata policy derives a sibling name by appending `.meta` to the complete data filename:

```text
audit.log → audit.log.meta
```

`metadata::check_user_access()` accepts reads and attribute queries. It returns `ReadOnlyMetadata` for user creation, writes, truncation, deletion, renaming, attribute changes, and link creation.

`metadata::check_user_entry_name()` reserves the `.meta` suffix, case-insensitively, for metadata entries. It also rejects empty names, `.` and `..`, path separators, and null bytes. `metadata::name_for()` applies these checks before deriving the metadata filename.

## FUSE namespace adapter

On Linux, `platform::fuse::NamespaceGuard` wraps a `fuser::Filesystem` implementation. Its `create`, `mknod`, `mkdir`, `symlink`, and `link` callbacks validate destination names before dispatching to the backing filesystem. Reserved `.meta` names return `EPERM`, including uppercase variants. Rename requests also return `EPERM`.

`NamespaceGuard::new(filesystem).mount(mountpoint, options)` mounts the wrapped filesystem and processes requests. Other callbacks forward to the backing implementation.

## macOS FSKit adapter

The macOS adapter uses Apple FSKit and the shared Rust namespace policy. It rejects reserved `.meta` creation and file, directory, and volume renames before dispatching to the backing volume. Native builds produce the FSKit adapter library alongside `ab-worm`.

## Verification

```sh
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo fmt --check
python tests/build_version.py
```

The policy tests cover fixed retention across appends, rejection of overwrites and gaps, deletion eligibility at the exact deadline, immutable file and directory paths, and arithmetic overflow.

The FUSE mount test exercises `fopen()`, directory creation, symbolic links, hard links, FIFO creation, and rename requests through FUSE. It checks both successful ordinary creation and rejected reserved names. On Linux, run it with access to `/dev/fuse` and FUSE mount permission:

```sh
cargo test --locked --test fuse_namespace -- --ignored
```

For macOS setup and native tests, see [macOS testing](docs/MACOS_TESTING.md):

```sh
./scripts/test-macos.sh
```

## FUSE references and licenses

FUSE and Rust `fuser` references, component licenses, and the original MIT notice are recorded in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

The Apple SDK, Rust/Swift runtime, WinFsp, winfsp-rs, and Dokany license review is recorded in [Filesystem dependency licenses](docs/DEPENDENCY_LICENSES.md).
