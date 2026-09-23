use super::durable::read_json;
use super::paths::{meta_path, regular};
use super::record;
use super::transaction::Transaction;
use super::{Record, Store, denied, invalid, now};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
};

impl Store {
    pub fn create(&mut self, name: &str, retention_seconds: u64) -> io::Result<Record> {
        self.recover()?;
        let path = self.data_path(name)?;
        let meta = meta_path(&path);
        if path.try_exists()? || meta.try_exists()? {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "File or metadata already exists",
            ));
        }
        let parent = path
            .parent()
            .ok_or_else(|| invalid("Storage path has no parent directory"))?;
        if !parent.is_dir() {
            return Err(invalid("Parent directory does not exist"));
        }
        let record = Record::new(now()?, retention_seconds)?;
        self.begin(&Transaction::Create {
            name: name.into(),
            record: record.clone(),
        })?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&path)?;
        file.sync_all()?;
        self.atomic_json(&meta, &record)?;
        self.finish()?;
        Ok(record)
    }

    pub fn metadata(&self, name: &str) -> io::Result<Record> {
        self.recover()?;
        let path = self.data_path(name)?;
        regular(&path)?;
        let record: Record = read_json(&meta_path(&path))?;
        record.hasher()?;
        if fs::metadata(path)?.len() != record.lock_offset {
            return Err(invalid("File size does not match LOCK"));
        }
        Ok(record)
    }

    pub fn append(&mut self, name: &str, data: &[u8]) -> io::Result<Record> {
        let offset = self.metadata(name)?.lock_offset;
        self.append_at(name, offset, data)
    }

    pub fn append_at(&mut self, name: &str, offset: u64, data: &[u8]) -> io::Result<Record> {
        let before = self.metadata(name)?;
        let after = before.append(offset, data)?;
        if data.is_empty() {
            return Ok(after);
        }
        let path = self.data_path(name)?;
        self.begin(&Transaction::Append {
            name: name.into(),
            before,
            after: after.clone(),
        })?;
        let mut file = OpenOptions::new().append(true).open(&path)?;
        file.write_all(data)?;
        file.sync_all()?;
        self.atomic_json(&meta_path(&path), &after)?;
        self.finish()?;
        Ok(after)
    }

    pub fn read(&self, name: &str, output: &mut impl Write) -> io::Result<u64> {
        let record = self.metadata(name)?;
        let mut file = File::open(self.data_path(name)?)?.take(record.lock_offset);
        io::copy(&mut file, output)
    }

    pub fn verify(&self, name: &str) -> io::Result<Record> {
        let record = self.metadata(name)?;
        let mut file = File::open(self.data_path(name)?)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            let chunk = buffer
                .get(..n)
                .ok_or_else(|| invalid("File read exceeded verification buffer"))?;
            hasher.update(chunk);
        }
        if record::hex(&hasher.finalize()) != record.checksum {
            return Err(invalid("File checksum mismatch"));
        }
        Ok(record)
    }

    pub fn delete(&mut self, name: &str) -> io::Result<()> {
        let record = self.metadata(name)?;
        if now()? < record.retain_until {
            return Err(denied("File retention is active"));
        }
        self.begin(&Transaction::Delete {
            name: name.into(),
            record,
        })?;
        self.recover()
    }
}
