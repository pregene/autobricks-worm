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

Linux and macOS:

```sh
./build.sh --release
./target/release/ab-worm --version
```

Windows (PowerShell):

```powershell
.\build.ps1 --release
.\target\release\ab-worm.exe --version
```

Rust selects the OS module at compile time:

| Target OS | Module | Executable |
| --- | --- | --- |
| Linux | `src/linux/` | `ab-worm` |
| macOS | `src/macos/` | `ab-worm` |
| Windows | `src/windows/` | `ab-worm.exe` |

The shared policy and metadata modules compile on all three targets. The Linux module contains the FUSE namespace adapter.

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

On Linux, `linux::fuse::NamespaceGuard` wraps a `fuser::Filesystem` implementation. Its `create`, `mknod`, `mkdir`, `symlink`, and `link` callbacks validate destination names before dispatching to the backing filesystem. Reserved `.meta` names return `EPERM`, including uppercase variants. Rename requests also return `EPERM`.

`NamespaceGuard::new(filesystem).mount(mountpoint, options)` mounts the wrapped filesystem and processes requests. Other callbacks forward to the backing implementation.

## Verification

```sh
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo fmt --check
python tests/build_version.py
```

The policy tests cover fixed retention across appends, rejection of overwrites and gaps, deletion eligibility at the exact deadline, immutable file and directory paths, and arithmetic overflow.

The Linux mount test exercises `fopen()`, directory creation, symbolic links, hard links, FIFO creation, and rename requests through FUSE. It checks both successful ordinary creation and rejected reserved names. Run it with access to `/dev/fuse` and FUSE mount permission:

```sh
cargo test --locked --test fuse_namespace -- --ignored
```

## Source layout

| Path | Purpose |
| --- | --- |
| `src/lib.rs` | Target OS selection and public module exports |
| `src/linux/mod.rs` | Linux configuration and FUSE integration |
| `src/macos/mod.rs` | macOS configuration |
| `src/windows/mod.rs` | Windows configuration |
| `src/policy/file.rs` | Append validation and retention deadlines |
| `src/policy/entry.rs` | Immutable file and directory paths |
| `src/policy/error.rs` | Policy error types |
| `src/metadata/names.rs` | Reserved names and metadata filename derivation |
| `src/metadata/access.rs` | Metadata user access rules |
| `src/linux/fuse/namespace.rs` | FUSE creation and rename enforcement |
| `src/linux/fuse/file_io.rs` | File I/O callback forwarding |
| `src/linux/fuse/directory_io.rs` | Directory I/O callback forwarding |
| `src/linux/fuse/attributes.rs` | Attribute and lookup callback forwarding |
| `src/linux/fuse/file_control.rs` | File control callback forwarding |
| `src/linux/fuse/lifecycle.rs` | Filesystem lifecycle callback forwarding |
| `src/linux/fuse/mod.rs` | FUSE adapter assembly and mount entry point |
| `tests/` | Policy tests and mounted FUSE integration tests |
| `tests/support/` | Test filesystem, mount cleanup, and syscall helpers |
| `src/main.rs` | Product banner and CLI argument handling |
| `build.rs` | Build-time version validation and embedding |
| `build.sh` | Linux and macOS build entry point |
| `build.ps1` | Windows PowerShell build entry point |
| `scripts/build_lock.py` | Native build locks for Unix and Windows |
| `scripts/build.py` | Version increments and build coordination |
| `VERSION` | Executable version |

## FUSE references and licenses

FUSE and Rust `fuser` references, component licenses, and the original MIT notice are recorded in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
