use super::{Store, denied, invalid};
use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

impl Store {
    pub(crate) fn data_path(&self, name: &str) -> io::Result<PathBuf> {
        if name.is_empty() || name.contains('\\') || name.contains(':') {
            return Err(invalid("Expected a relative storage path"));
        }
        let mut path = self.root.clone();
        for part in Path::new(name).components() {
            let Component::Normal(value) = part else {
                return Err(invalid("Invalid storage path"));
            };
            crate::metadata::check_user_entry_name(value)
                .map_err(|_| denied("Reserved or invalid entry name"))?;
            if value
                .to_string_lossy()
                .to_ascii_lowercase()
                .starts_with(".ab-worm")
            {
                return Err(denied("Reserved storage control name"));
            }
            path.push(value);
            match fs::symlink_metadata(&path) {
                Ok(info) => {
                    if info.file_type().is_symlink() {
                        return Err(denied("Storage symlinks are forbidden"));
                    }
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::MetadataExt;
                        if info.is_file() && info.nlink() != 1 {
                            return Err(denied("Storage hard links are forbidden"));
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
        }
        Ok(path)
    }
}

pub(crate) fn meta_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".meta");
    name.into()
}

pub(crate) fn regular(path: &Path) -> io::Result<()> {
    let info = fs::symlink_metadata(path)?;
    if !info.is_file() {
        return Err(denied("Expected a regular storage file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if info.nlink() != 1 {
            return Err(denied("Storage hard links are forbidden"));
        }
    }
    Ok(())
}

pub(crate) fn private_dir(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
