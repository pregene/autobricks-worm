use super::paths::regular;
use super::{Store, invalid};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

pub(crate) fn sync_dir(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()?;
    }
    Ok(())
}

pub(crate) fn read_json<T: DeserializeOwned>(path: &Path) -> io::Result<T> {
    regular(path)?;
    let mut data = Vec::new();
    File::open(path)?.take(16385).read_to_end(&mut data)?;
    if data.len() > 16384 {
        return Err(invalid("Metadata exceeds size limit"));
    }
    serde_json::from_slice(&data).map_err(|_| invalid("Invalid metadata JSON"))
}

impl Store {
    pub(crate) fn atomic_json(&self, path: &Path, value: &impl Serialize) -> io::Result<()> {
        let stage = self.root.join(".ab-worm.stage");
        // Only one transaction can run while this Store owns the storage lock.
        if stage.try_exists()? {
            regular(&stage)?;
            fs::remove_file(&stage)?;
        }
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o400);
        }
        let mut file = options.open(&stage)?;
        serde_json::to_writer_pretty(&mut file, value).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        drop(file);
        fs::rename(&stage, path)?;
        sync_dir(path.parent().unwrap())?;
        if path.parent() != Some(self.root.as_path()) {
            sync_dir(&self.root)?;
        }
        Ok(())
    }
}
