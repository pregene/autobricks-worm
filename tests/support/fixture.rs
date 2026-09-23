use fuser::*;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

#[derive(Default)]
pub struct State {
    entries: HashMap<(u64, OsString), FileAttr>,
    next_inode: u64,
    pub creation_calls: Vec<OsString>,
    pub rename_calls: usize,
}

pub struct Fixture(pub Arc<Mutex<State>>);

fn attr(ino: u64, kind: FileType) -> FileAttr {
    FileAttr {
        ino,
        size: 0,
        blocks: 0,
        atime: UNIX_EPOCH,
        mtime: UNIX_EPOCH,
        ctime: UNIX_EPOCH,
        crtime: UNIX_EPOCH,
        kind,
        perm: 0o777,
        nlink: 1,
        uid: 0,
        gid: 0,
        rdev: 0,
        blksize: 4096,
        flags: 0,
    }
}

impl Fixture {
    fn add(&self, parent: u64, name: &OsStr, kind: FileType) -> FileAttr {
        let mut state = self.0.lock().unwrap();
        state.creation_calls.push(name.to_owned());
        state.next_inode += 1;
        let entry = attr(state.next_inode + 1, kind);
        state.entries.insert((parent, name.to_owned()), entry);
        entry
    }
}

impl Filesystem for Fixture {
    fn lookup(&mut self, _: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self
            .0
            .lock()
            .unwrap()
            .entries
            .get(&(parent, name.to_owned()))
        {
            Some(entry) => reply.entry(&Duration::ZERO, entry, 0),
            None => reply.error(libc::ENOENT),
        }
    }

    fn getattr(&mut self, _: &Request<'_>, ino: u64, _: Option<u64>, reply: ReplyAttr) {
        if ino == 1 {
            reply.attr(&Duration::ZERO, &attr(1, FileType::Directory));
            return;
        }
        match self
            .0
            .lock()
            .unwrap()
            .entries
            .values()
            .find(|entry| entry.ino == ino)
        {
            Some(entry) => reply.attr(&Duration::ZERO, entry),
            None => reply.error(libc::ENOENT),
        }
    }

    fn create(
        &mut self,
        _: &Request<'_>,
        parent: u64,
        name: &OsStr,
        _: u32,
        _: u32,
        _: i32,
        reply: ReplyCreate,
    ) {
        let entry = self.add(parent, name, FileType::RegularFile);
        reply.created(&Duration::ZERO, &entry, 0, 0, 0);
    }

    fn mknod(
        &mut self,
        _: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _: u32,
        _: u32,
        reply: ReplyEntry,
    ) {
        let kind = if u64::from(mode) & u64::from(libc::S_IFMT) == u64::from(libc::S_IFIFO) {
            FileType::NamedPipe
        } else {
            FileType::RegularFile
        };
        let entry = self.add(parent, name, kind);
        reply.entry(&Duration::ZERO, &entry, 0);
    }

    fn mkdir(
        &mut self,
        _: &Request<'_>,
        parent: u64,
        name: &OsStr,
        _: u32,
        _: u32,
        reply: ReplyEntry,
    ) {
        let entry = self.add(parent, name, FileType::Directory);
        reply.entry(&Duration::ZERO, &entry, 0);
    }

    fn symlink(&mut self, _: &Request<'_>, parent: u64, name: &OsStr, _: &Path, reply: ReplyEntry) {
        let entry = self.add(parent, name, FileType::Symlink);
        reply.entry(&Duration::ZERO, &entry, 0);
    }

    fn link(&mut self, _: &Request<'_>, ino: u64, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let mut state = self.0.lock().unwrap();
        let entry = *state
            .entries
            .values()
            .find(|entry| entry.ino == ino)
            .unwrap();
        state.creation_calls.push(name.to_owned());
        state.entries.insert((parent, name.to_owned()), entry);
        reply.entry(&Duration::ZERO, &entry, 0);
    }

    fn rename(
        &mut self,
        _: &Request<'_>,
        _: u64,
        _: &OsStr,
        _: u64,
        _: &OsStr,
        _: u32,
        reply: ReplyEmpty,
    ) {
        self.0.lock().unwrap().rename_calls += 1;
        reply.ok();
    }

    fn flush(&mut self, _: &Request<'_>, _: u64, _: u64, _: u64, reply: ReplyEmpty) {
        reply.ok();
    }
}
