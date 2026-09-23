use super::durable::read_json;
use super::invalid;
use super::paths::{private_dir, regular};
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

/// Keep this handle alive for the lifetime of a mounted volume or storage command.
/// Backing files are private to the service account.
pub struct Store {
    pub(crate) root: PathBuf,
    _lock: File,
    _exclusive: std::marker::PhantomData<std::cell::Cell<()>>,
}

impl Store {
    pub fn open(root: &Path) -> io::Result<Self> {
        let info = fs::symlink_metadata(root)?;
        if !info.is_dir() || info.file_type().is_symlink() {
            return Err(invalid("Storage root must be an existing directory"));
        }
        let root = root.canonicalize()?;
        let marker = root.join(".ab-worm.store");
        if marker.try_exists()? {
            let format: String = read_json(&marker)?;
            if format != "Autobricks WORM storage v1" {
                return Err(invalid("Unsupported storage format"));
            }
        } else {
            for entry in fs::read_dir(&root)? {
                if entry?.file_name() != ".ab-worm.lock" {
                    return Err(invalid("Initialize storage in an empty directory"));
                }
            }
        }
        private_dir(&root)?;
        let path = root.join(".ab-worm.lock");
        if path.try_exists()? {
            regular(&path)?;
        }
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        let lock = options.open(path)?;
        lock.try_lock_exclusive().map_err(|e| {
            io::Error::new(e.kind(), "Storage is already in use or cannot be locked")
        })?;
        let store = Self {
            root,
            _lock: lock,
            _exclusive: std::marker::PhantomData,
        };
        if !marker.try_exists()? {
            store.atomic_json(&marker, &"Autobricks WORM storage v1")?;
        }
        store.recover()?;
        Ok(store)
    }
}
