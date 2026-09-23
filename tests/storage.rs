use autobricks_worm::storage::{Record, Store};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "ab-worm-storage-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn replace(path: &Path, data: &[u8]) {
    let temp = path.with_extension("replacement");
    fs::write(&temp, data).unwrap();
    fs::rename(temp, path).unwrap();
}
fn checksum(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn persistent_appends_match_standard_sha256_across_restart_and_block_boundaries() {
    let dir = Temp::new();
    let initial = Store::open(&dir.0)
        .unwrap()
        .create("audit.log", 3600)
        .unwrap();
    assert_eq!(initial.checksum, checksum(b""));
    let mut content = Vec::new();
    for length in [16, 16, 31, 1, 1, 63, 64, 65, 4096] {
        let buffer: Vec<_> = (0..length).map(|i| (i % 251) as u8).collect();
        content.extend_from_slice(&buffer);
        let mut store = Store::open(&dir.0).unwrap();
        let meta = store.append("audit.log", &buffer).unwrap();
        assert_eq!(meta.checksum, checksum(&content));
        assert_eq!(meta.created_at, initial.created_at);
        assert_eq!(meta.retain_until, initial.retain_until);
        assert_eq!(meta.lock_offset, content.len() as u64);
        assert_eq!(meta.sha256_state.len(), initial.sha256_state.len());
        assert!(fs::metadata(dir.0.join("audit.log.meta")).unwrap().len() < 2048);
        assert_eq!(store.verify("audit.log").unwrap(), meta);
    }
    assert_eq!(fs::read(dir.0.join("audit.log")).unwrap(), content);
}

#[test]
fn overwrite_retention_reserved_names_and_concurrent_access_are_rejected() {
    let dir = Temp::new();
    let mut store = Store::open(&dir.0).unwrap();
    store.create("retained", 3600).unwrap();
    store.append("retained", b"first").unwrap();
    assert!(store.append_at("retained", 0, b"replace").is_err());
    assert!(store.append_at("retained", 7, b"gap").is_err());
    assert_eq!(
        store.delete("retained").unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    for name in [
        "data.meta",
        "DATA.META",
        "../escape",
        "/absolute",
        ".ab-worm.lock",
        "a\\b",
    ] {
        assert!(store.create(name, 0).is_err(), "{name}");
        assert!(store.mkdir(name).is_err(), "{name}");
    }
    assert!(store.create("retained", 0).is_err());
    assert!(Store::open(&dir.0).is_err());
    store.create("expired", 0).unwrap();
    store.append("expired", b"still immutable").unwrap();
    assert!(store.append_at("expired", 0, b"x").is_err());
    store.delete("expired").unwrap();
    assert!(!dir.0.join("expired").exists());
    assert!(!dir.0.join("expired.meta").exists());
    store.mkdir("folder").unwrap();
    store.create("folder/nested", 0).unwrap();
    store.append("folder/nested", b"nested").unwrap();
    store.verify("folder/nested").unwrap();
}

#[test]
fn corruption_fails_verification_without_rewriting_the_file() {
    let dir = Temp::new();
    let mut store = Store::open(&dir.0).unwrap();
    store.create("data", 0).unwrap();
    store.append("data", b"original").unwrap();
    fs::write(dir.0.join("data"), b"tampered").unwrap();
    assert!(store.verify("data").is_err());
    let mut corrupt = store.metadata("data").unwrap();
    corrupt.lock_offset += 1;
    replace(
        &dir.0.join("data.meta"),
        &serde_json::to_vec(&corrupt).unwrap(),
    );
    assert!(store.append("data", b"more").is_err());
    assert_eq!(fs::read(dir.0.join("data")).unwrap(), b"tampered");
}

#[test]
fn incomplete_append_rolls_back_only_uncommitted_tail_and_committed_append_survives() {
    let dir = Temp::new();
    let (before, after) = {
        let mut store = Store::open(&dir.0).unwrap();
        store.create("data", 3600).unwrap();
        let before = store.append("data", b"first").unwrap();
        let after = store.append("data", b"second").unwrap();
        (before, after)
    };
    let journal =
        serde_json::json!({"operation":"Append", "name":"data", "before":before, "after":after});
    fs::write(dir.0.join(".ab-worm.transaction"), journal.to_string()).unwrap();
    // Data and metadata committed, but transaction cleanup was interrupted.
    assert_eq!(Store::open(&dir.0).unwrap().verify("data").unwrap(), after);
    // Partial data reached disk, while metadata still refers to the old LOCK.
    fs::write(dir.0.join("data"), b"firstsec").unwrap();
    replace(
        &dir.0.join("data.meta"),
        &serde_json::to_vec(&before).unwrap(),
    );
    fs::write(dir.0.join(".ab-worm.transaction"), journal.to_string()).unwrap();
    let store = Store::open(&dir.0).unwrap();
    assert_eq!(store.verify("data").unwrap(), before);
    assert_eq!(fs::read(dir.0.join("data")).unwrap(), b"first");
}

#[test]
fn interrupted_create_and_delete_are_recovered() {
    let dir = Temp::new();
    let record = {
        let mut store = Store::open(&dir.0).unwrap();
        store.create("data", 0).unwrap()
    };
    fs::remove_file(dir.0.join("data.meta")).unwrap();
    let journal = serde_json::json!({"operation":"Create", "name":"data", "record":record});
    fs::write(dir.0.join(".ab-worm.transaction"), journal.to_string()).unwrap();
    let mut store = Store::open(&dir.0).unwrap();
    assert!(!dir.0.join("data").exists());
    let record = store.create("data", 0).unwrap();
    drop(store);
    fs::remove_file(dir.0.join("data")).unwrap();
    let journal = serde_json::json!({"operation":"Delete", "name":"data", "record":record});
    fs::write(dir.0.join(".ab-worm.transaction"), journal.to_string()).unwrap();
    let _store = Store::open(&dir.0).unwrap();
    assert!(!dir.0.join("data.meta").exists());
}

#[cfg(unix)]
#[test]
fn storage_paths_reject_symlink_and_hardlink_escape() {
    use std::os::unix::fs::symlink;
    let dir = Temp::new();
    let external = Temp::new();
    let mut store = Store::open(&dir.0).unwrap();
    symlink(&external.0, dir.0.join("linked")).unwrap();
    assert!(store.create("linked/escape", 0).is_err());
    store.create("data", 0).unwrap();
    fs::hard_link(dir.0.join("data"), external.0.join("alias")).unwrap();
    assert!(store.append("data", b"x").is_err());
    assert_eq!(fs::read(external.0.join("alias")).unwrap(), b"");
}

#[test]
fn cli_streams_input_and_reads_exact_bytes() {
    let dir = Temp::new();
    let command = |operation: &str| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ab-worm"));
        command.args(["storage", dir.0.to_str().unwrap(), operation, "data"]);
        command
    };
    let result = command("create").arg("0").output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut child = command("append")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"hello\x00world")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let record: Record = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(record.checksum, checksum(b"hello\x00world"));
    let output = command("read").output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"hello\x00world");
    assert!(command("verify").output().unwrap().status.success());
}
