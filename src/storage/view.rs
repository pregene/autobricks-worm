//! Read-only namespace view used by mounted filesystem adapters.
use super::paths::{meta_path, regular};
use super::{Record, Store, denied, invalid};
use serde::Serialize;
use std::{
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom},
    path::PathBuf,
    time::UNIX_EPOCH,
};

#[derive(Serialize)]
pub struct Entry {
    pub name: String,
    pub directory: bool,
    pub metadata: bool,
    pub size: u64,
    pub created_at: u64,
    pub modified_at: u64,
}

impl Store {
    fn view_path(&self, name: &str) -> io::Result<(PathBuf, bool, Option<Record>)> {
        if name.is_empty() {
            return Ok((self.root.clone(), false, None));
        }
        if let Some(data_name) = name.strip_suffix(".meta") {
            let record = self.metadata(data_name)?;
            let path = meta_path(&self.data_path(data_name)?);
            regular(&path)?;
            return Ok((path, true, Some(record)));
        }
        let path = self.data_path(name)?;
        if fs::symlink_metadata(&path)?.is_dir() {
            return Ok((path, false, None));
        }
        let record = self.metadata(name)?;
        Ok((path, false, Some(record)))
    }

    pub fn entry(&self, name: &str) -> io::Result<Entry> {
        let (path, metadata, record) = self.view_path(name)?;
        let info = fs::symlink_metadata(path)?;
        let seconds = |time: io::Result<std::time::SystemTime>| {
            time.ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |time| time.as_secs())
        };
        Ok(Entry {
            name: name.into(),
            directory: info.is_dir(),
            metadata,
            size: info.len(),
            created_at: record.map_or_else(|| seconds(info.created()), |record| record.created_at),
            modified_at: seconds(info.modified()),
        })
    }

    pub fn entries(&self, name: &str) -> io::Result<Vec<Entry>> {
        let (path, _, _) = self.view_path(name)?;
        if !fs::metadata(&path)?.is_dir() {
            return Err(io::Error::from(io::ErrorKind::NotADirectory));
        }
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            let file_name = entry?
                .file_name()
                .into_string()
                .map_err(|_| invalid("Filename must be UTF-8"))?;
            if file_name.to_ascii_lowercase().starts_with(".ab-worm") {
                continue;
            }
            let relative = if name.is_empty() {
                file_name
            } else {
                format!("{name}/{file_name}")
            };
            entries.push(self.entry(&relative)?);
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(entries)
    }

    pub fn read_at(&self, name: &str, offset: u64, length: usize) -> io::Result<Vec<u8>> {
        let (path, _, _) = self.view_path(name)?;
        regular(&path)?;
        let mut file = File::open(path)?;
        file.seek(SeekFrom::Start(offset))?;
        let mut result = Vec::new();
        file.take(length.min(1024 * 1024) as u64)
            .read_to_end(&mut result)?;
        Ok(result)
    }

    pub fn remove_entry(&mut self, name: &str) -> io::Result<()> {
        if self.entry(name)?.directory {
            return Err(denied("Directory deletion is forbidden"));
        }
        self.delete(name)
    }
}
