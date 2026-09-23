#![cfg(target_os = "linux")]

mod support;

use autobricks_worm::linux::fuse::NamespaceGuard;
use fuser::MountOption;
use std::fs;
use std::os::unix::fs::symlink;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use support::fixture::{Fixture, State};
use support::mount::Mount;
use support::syscalls::{fopen, mkfifo};

fn assert_denied(result: std::io::Result<()>) {
    assert_eq!(result.unwrap_err().raw_os_error(), Some(libc::EPERM));
}

#[test]
#[ignore = "Requires Linux /dev/fuse and permission to mount a FUSE filesystem"]
fn mounted_namespace_rejects_reserved_names_before_backend_dispatch() {
    let path = std::env::temp_dir().join(format!(
        "ab-worm-fuse-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    let state = Arc::new(Mutex::new(State::default()));
    let session = fuser::spawn_mount2(
        NamespaceGuard::new(Fixture(state.clone())),
        &path,
        &[MountOption::FSName("ab-worm-test".into())],
    )
    .expect("mount FUSE test filesystem");
    let mount = Mount {
        session: Some(session),
        path,
    };
    let root = &mount.path;

    // Positive controls: the backing fixture accepts ordinary creation paths.
    fopen(&root.join("audit.log")).unwrap();
    fs::create_dir(root.join("folder")).unwrap();
    symlink("audit.log", root.join("alias")).unwrap();
    fs::hard_link(root.join("audit.log"), root.join("hardlink")).unwrap();
    mkfifo(&root.join("pipe")).unwrap();
    let allowed_calls = state.lock().unwrap().creation_calls.len();
    assert_eq!(allowed_calls, 5);

    // Kernel-dispatched CREATE, MKDIR, SYMLINK, LINK and MKNOD requests.
    for name in ["audit.log.meta", "audit.META", ".meta", "missing.meta"] {
        assert_denied(fopen(&root.join(name)));
        assert_denied(fs::create_dir(root.join(name)));
        assert_denied(symlink("audit.log", root.join(name)));
        assert_denied(fs::hard_link(root.join("audit.log"), root.join(name)));
        assert_denied(mkfifo(&root.join(name)));
        assert!(!root.join(name).exists());
    }
    assert_denied(fopen(&root.join("folder/nested.meta")));
    assert_denied(fs::rename(
        root.join("audit.log"),
        root.join("renamed.meta"),
    ));
    assert_denied(fs::rename(root.join("audit.log"), root.join("renamed.log")));
    let state = state.lock().unwrap();
    assert_eq!(state.creation_calls.len(), allowed_calls);
    assert_eq!(state.rename_calls, 0);
}
