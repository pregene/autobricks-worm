use super::durable::{read_json, sync_dir};
use super::paths::{meta_path, regular};
use super::{Record, Store, invalid};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io,
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "operation", deny_unknown_fields)]
pub(crate) enum Transaction {
    Create {
        name: String,
        record: Record,
    },
    Append {
        name: String,
        before: Record,
        after: Record,
    },
    Delete {
        name: String,
        record: Record,
    },
}

impl Store {
    pub(crate) fn begin(&self, transaction: &Transaction) -> io::Result<()> {
        self.atomic_json(&self.root.join(".ab-worm.transaction"), transaction)
    }

    pub(crate) fn finish(&self) -> io::Result<()> {
        fs::remove_file(self.root.join(".ab-worm.transaction"))?;
        sync_dir(&self.root)
    }

    pub(crate) fn recover(&self) -> io::Result<()> {
        let journal = self.root.join(".ab-worm.transaction");
        if !journal.try_exists()? {
            return Ok(());
        }
        let transaction: Transaction = read_json(&journal)?;
        match transaction {
            Transaction::Create { name, record } => {
                record.hasher()?;
                if record.lock_offset != 0 {
                    return Err(invalid("Invalid create transaction"));
                }
                let path = self.data_path(&name)?;
                let meta = meta_path(&path);
                if meta.try_exists()? {
                    let stored: Record = read_json(&meta)?;
                    regular(&path)?;
                    if stored != record || fs::metadata(&path)?.len() != 0 {
                        return Err(invalid("Incomplete create has conflicting data"));
                    }
                } else if path.try_exists()? {
                    regular(&path)?;
                    if fs::metadata(&path)?.len() != 0 {
                        return Err(invalid("Incomplete create contains data"));
                    }
                    fs::remove_file(&path)?;
                }
                sync_dir(path.parent().unwrap())?;
            }
            Transaction::Append {
                name,
                before,
                after,
            } => {
                before.hasher()?;
                after.hasher()?;
                if after.lock_offset <= before.lock_offset
                    || before.created_at != after.created_at
                    || before.retain_until != after.retain_until
                {
                    return Err(invalid("Invalid append transaction"));
                }
                let path = self.data_path(&name)?;
                regular(&path)?;
                let current: Record = read_json(&meta_path(&path))?;
                let length = fs::metadata(&path)?.len();
                if current == before && length >= before.lock_offset {
                    let file = OpenOptions::new().write(true).open(&path)?;
                    file.set_len(before.lock_offset)?;
                    file.sync_all()?;
                } else if current != after || length != after.lock_offset {
                    return Err(invalid(
                        "Append recovery found inconsistent data and metadata",
                    ));
                }
            }
            Transaction::Delete { name, record } => {
                record.hasher()?;
                let path = self.data_path(&name)?;
                let meta = meta_path(&path);
                if meta.try_exists()? {
                    let stored: Record = read_json(&meta)?;
                    if stored != record {
                        return Err(invalid("Delete recovery found conflicting metadata"));
                    }
                }
                if path.try_exists()? {
                    regular(&path)?;
                    fs::remove_file(&path)?;
                }
                if meta.try_exists()? {
                    fs::remove_file(&meta)?;
                }
                sync_dir(path.parent().unwrap())?;
            }
        }
        self.finish()
    }
}
