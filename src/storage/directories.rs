use super::Store;
use super::durable::sync_dir;
use super::paths::private_dir;
use std::{fs, io};

impl Store {
    pub fn mkdir(&mut self, name: &str) -> io::Result<()> {
        self.recover()?;
        let path = self.data_path(name)?;
        fs::create_dir(&path)?;
        private_dir(&path)?;
        sync_dir(path.parent().unwrap())
    }
}
