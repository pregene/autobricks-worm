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

Requirements: Rust/Cargo and Python 3 on Linux or macOS.

```sh
./build.sh --release
./target/release/ab-worm --version
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

`EntryPolicy` represents a file or directory. Its `check_rename()` policy rejects name changes and moves with `ImmutablePath`, including replacement and exchange operations. A created entry retains its name and parent directory, including after file retention expires.

## Verification

```sh
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo fmt --check
```

The policy tests cover fixed retention across appends, rejection of overwrites and gaps, deletion eligibility at the exact deadline, immutable file and directory paths, and arithmetic overflow.

## Source layout

| Path | Purpose |
| --- | --- |
| `src/lib.rs` | WORM policy calculations and tests |
| `src/main.rs` | Product banner and CLI argument handling |
| `build.rs` | Build-time version validation and embedding |
| `build.sh` | Build entry point |
| `scripts/build.py` | Version increments and build coordination |
| `VERSION` | Executable version |

## FUSE references and licenses

FUSE and Rust `fuser` references, component licenses, and the original MIT notice are recorded in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
