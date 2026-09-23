use fuser::BackgroundSession;
use std::fs;
use std::path::PathBuf;

pub struct Mount {
    pub session: Option<BackgroundSession>,
    pub path: PathBuf,
}

impl Drop for Mount {
    fn drop(&mut self) {
        drop(self.session.take());
        fs::remove_dir(&self.path).expect("remove test mountpoint");
    }
}
