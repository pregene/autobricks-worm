use crate::cli::mount::MountOptions;
use crate::fuse::NamespaceGuard;
use crate::storage::{Entry, Store};
use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory,
    ReplyEmpty, ReplyEntry, ReplyOpen, ReplyWrite, Request, TimeOrNow, consts,
};
use libc::{EACCES, EINVAL, EIO, ENOENT, ENOSYS, ENOTDIR, EPERM, EROFS, c_int};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, UNIX_EPOCH};

const TTL: Duration = Duration::from_secs(0);
const ROOT_INO: u64 = 1;

pub fn mount(options: MountOptions) -> io::Result<()> {
    if !options.source.exists() {
        fs::create_dir_all(&options.source)?;
    }
    if !options.mountpoint.exists() {
        fs::create_dir_all(&options.mountpoint)?;
    }
    let source = options.source.canonicalize()?;
    let mountpoint = options.mountpoint.canonicalize()?;
    if source.starts_with(&mountpoint) || mountpoint.starts_with(&source) {
        return Err(io::Error::other(
            "Source and mountpoint must be separate, non-nested directories",
        ));
    }
    if !fs::symlink_metadata(&mountpoint)?.is_dir() {
        return Err(io::Error::other("Mountpoint must be a directory"));
    }
    if fs::read_dir(&mountpoint)?.next().is_some() {
        return Err(io::Error::other("Mountpoint must be empty"));
    }

    let retention_seconds = options
        .retention_days
        .checked_mul(86400)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Retention days overflow"))?;
    let store = Store::open(&source)?;
    let filesystem = StorageFilesystem::new(store, retention_seconds);
    let mut mount_options = vec![
        MountOption::FSName("ab-worm".into()),
        MountOption::Subtype("abworm".into()),
    ];
    if options.allow_other {
        mount_options.push(MountOption::AllowOther);
    }
    eprintln!(
        "Mounting {} at {} (new-file retention: {} days)",
        source.display(),
        mountpoint.display(),
        options.retention_days
    );
    NamespaceGuard::new(filesystem).mount(&mountpoint, &mount_options)
}

pub fn unmount(path: &Path) -> io::Result<()> {
    let status = Command::new("fusermount3")
        .arg("-u")
        .arg(path)
        .status()
        .or_else(|_| Command::new("fusermount").arg("-u").arg(path).status())
        .or_else(|_| Command::new("umount").arg(path).status())?;
    if status.success() {
        println!("Unmounted {}", path.display());
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "Unmount command failed with status {status}"
        )))
    }
}

struct StorageFilesystem {
    store: Store,
    retention_seconds: u64,
    next_inode: u64,
    paths_by_inode: HashMap<u64, String>,
    inodes_by_path: HashMap<String, u64>,
}

impl StorageFilesystem {
    fn new(store: Store, retention_seconds: u64) -> Self {
        let mut paths_by_inode = HashMap::new();
        let mut inodes_by_path = HashMap::new();
        paths_by_inode.insert(ROOT_INO, String::new());
        inodes_by_path.insert(String::new(), ROOT_INO);
        Self {
            store,
            retention_seconds,
            next_inode: ROOT_INO + 1,
            paths_by_inode,
            inodes_by_path,
        }
    }

    fn path_for(&self, ino: u64) -> io::Result<&str> {
        self.paths_by_inode
            .get(&ino)
            .map(String::as_str)
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }

    fn inode_for(&mut self, path: &str) -> io::Result<u64> {
        if let Some(ino) = self.inodes_by_path.get(path) {
            return Ok(*ino);
        }
        let ino = self.next_inode;
        self.next_inode = self
            .next_inode
            .checked_add(1)
            .ok_or_else(|| io::Error::other("Inode space exhausted"))?;
        self.paths_by_inode.insert(ino, path.to_string());
        self.inodes_by_path.insert(path.to_string(), ino);
        Ok(ino)
    }

    fn child_path(&self, parent: u64, name: &OsStr) -> io::Result<String> {
        let name = name
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Filename must be UTF-8"))?;
        let parent = self.path_for(parent)?;
        Ok(if parent.is_empty() {
            name.to_string()
        } else {
            format!("{parent}/{name}")
        })
    }

    fn parent_inode(&mut self, path: &str) -> io::Result<u64> {
        let Some((parent, _)) = path.rsplit_once('/') else {
            return Ok(ROOT_INO);
        };
        self.inode_for(parent)
    }

    fn attr_for_path(&mut self, path: &str) -> io::Result<FileAttr> {
        let entry = self.store.entry(path)?;
        let ino = self.inode_for(path)?;
        attr(ino, &entry)
    }

    fn reply_entry_for_path(&mut self, path: &str, reply: ReplyEntry) {
        match self.attr_for_path(path) {
            Ok(attr) => reply.entry(&TTL, &attr, 0),
            Err(error) => reply.error(errno(&error)),
        }
    }
}

impl Filesystem for StorageFilesystem {
    fn lookup(&mut self, _: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self.child_path(parent, name) {
            Ok(path) => self.reply_entry_for_path(&path, reply),
            Err(error) => reply.error(errno(&error)),
        }
    }

    fn getattr(&mut self, _: &Request<'_>, ino: u64, _: Option<u64>, reply: ReplyAttr) {
        let result = self.path_for(ino).map(str::to_string);
        match result.and_then(|path| self.attr_for_path(&path)) {
            Ok(attr) => reply.attr(&TTL, &attr),
            Err(error) => reply.error(errno(&error)),
        }
    }

    fn setattr(
        &mut self,
        _: &Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        _: Option<TimeOrNow>,
        _: Option<TimeOrNow>,
        _: Option<std::time::SystemTime>,
        _: Option<u64>,
        _: Option<std::time::SystemTime>,
        _: Option<std::time::SystemTime>,
        _: Option<std::time::SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        if mode.is_some() || uid.is_some() || gid.is_some() || flags.is_some() {
            reply.error(EPERM);
            return;
        }
        let result = self
            .path_for(ino)
            .map(str::to_string)
            .and_then(|path| self.attr_for_path(&path));
        match (result, size) {
            (Ok(attr), Some(size)) if size == attr.size => reply.attr(&TTL, &attr),
            (Ok(_), Some(_)) => reply.error(EPERM),
            (Ok(attr), None) => reply.attr(&TTL, &attr),
            (Err(error), _) => reply.error(errno(&error)),
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
        let result = self
            .child_path(parent, name)
            .and_then(|path| {
                self.store
                    .create(&path, self.retention_seconds)
                    .map(|_| path)
            })
            .and_then(|path| self.attr_for_path(&path));
        match result {
            Ok(attr) => reply.created(&TTL, &attr, 0, 0, consts::FOPEN_DIRECT_IO),
            Err(error) => reply.error(errno(&error)),
        }
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
        if mode & libc::S_IFMT != libc::S_IFREG {
            reply.error(EPERM);
            return;
        }
        let result = self
            .child_path(parent, name)
            .and_then(|path| {
                self.store
                    .create(&path, self.retention_seconds)
                    .map(|_| path)
            })
            .and_then(|path| self.attr_for_path(&path));
        match result {
            Ok(attr) => reply.entry(&TTL, &attr, 0),
            Err(error) => reply.error(errno(&error)),
        }
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
        let result = self
            .child_path(parent, name)
            .and_then(|path| self.store.mkdir(&path).map(|_| path))
            .and_then(|path| self.attr_for_path(&path));
        match result {
            Ok(attr) => reply.entry(&TTL, &attr, 0),
            Err(error) => reply.error(errno(&error)),
        }
    }

    fn open(&mut self, _: &Request<'_>, ino: u64, flags: i32, reply: ReplyOpen) {
        let access_mode = flags & libc::O_ACCMODE;
        let result = self
            .path_for(ino)
            .map(str::to_string)
            .and_then(|path| self.store.entry(&path).map(|entry| (path, entry)));
        match result {
            Ok((_, entry)) if entry.directory => reply.error(EISDIR),
            Ok((_, entry)) if entry.metadata && access_mode != libc::O_RDONLY => reply.error(EROFS),
            Ok(_) => reply.opened(0, consts::FOPEN_DIRECT_IO),
            Err(error) => reply.error(errno(&error)),
        }
    }

    fn read(
        &mut self,
        _: &Request<'_>,
        ino: u64,
        _: u64,
        offset: i64,
        size: u32,
        _: i32,
        _: Option<u64>,
        reply: ReplyData,
    ) {
        if offset < 0 {
            reply.error(EINVAL);
            return;
        }
        let result = self
            .path_for(ino)
            .map(str::to_string)
            .and_then(|path| self.store.read_at(&path, offset as u64, size as usize));
        match result {
            Ok(data) => reply.data(&data),
            Err(error) => reply.error(errno(&error)),
        }
    }

    fn write(
        &mut self,
        _: &Request<'_>,
        ino: u64,
        _: u64,
        offset: i64,
        data: &[u8],
        _: u32,
        _: i32,
        _: Option<u64>,
        reply: ReplyWrite,
    ) {
        if offset < 0 {
            reply.error(EINVAL);
            return;
        }
        let result = self.path_for(ino).map(str::to_string).and_then(|path| {
            if path.ends_with(".meta") {
                return Err(io::Error::from(io::ErrorKind::PermissionDenied));
            }
            self.store.append_at(&path, offset as u64, data)
        });
        match result {
            Ok(_) => reply.written(data.len() as u32),
            Err(error) => reply.error(errno(&error)),
        }
    }

    fn flush(&mut self, _: &Request<'_>, _: u64, _: u64, _: u64, reply: ReplyEmpty) {
        reply.ok();
    }

    fn fsync(&mut self, _: &Request<'_>, _: u64, _: u64, _: bool, reply: ReplyEmpty) {
        reply.ok();
    }

    fn readdir(
        &mut self,
        _: &Request<'_>,
        ino: u64,
        _: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let path = match self.path_for(ino).map(str::to_string) {
            Ok(path) => path,
            Err(error) => {
                reply.error(errno(&error));
                return;
            }
        };
        let parent = match self.parent_inode(&path) {
            Ok(parent) => parent,
            Err(error) => {
                reply.error(errno(&error));
                return;
            }
        };
        let mut rows = vec![
            (ino, FileType::Directory, ".".to_string()),
            (parent, FileType::Directory, "..".to_string()),
        ];
        match self.store.entries(&path) {
            Ok(entries) => {
                for entry in entries {
                    let ino = match self.inode_for(&entry.name) {
                        Ok(ino) => ino,
                        Err(error) => {
                            reply.error(errno(&error));
                            return;
                        }
                    };
                    let kind = if entry.directory {
                        FileType::Directory
                    } else {
                        FileType::RegularFile
                    };
                    let name = entry
                        .name
                        .rsplit('/')
                        .next()
                        .unwrap_or(&entry.name)
                        .to_string();
                    rows.push((ino, kind, name));
                }
            }
            Err(error) => {
                reply.error(errno(&error));
                return;
            }
        }
        for (index, (ino, kind, name)) in rows.into_iter().enumerate().skip(offset.max(0) as usize)
        {
            let Some(next_offset) = i64::try_from(index)
                .ok()
                .and_then(|value| value.checked_add(1))
            else {
                reply.error(EIO);
                return;
            };
            if reply.add(ino, next_offset, kind, name) {
                break;
            }
        }
        reply.ok();
    }

    fn unlink(&mut self, _: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let result = self
            .child_path(parent, name)
            .and_then(|path| self.store.remove_entry(&path).map(|_| path));
        match result {
            Ok(path) => {
                if let Some(ino) = self.inodes_by_path.remove(&path) {
                    self.paths_by_inode.remove(&ino);
                }
                reply.ok();
            }
            Err(error) => reply.error(errno(&error)),
        }
    }

    fn rmdir(&mut self, _: &Request<'_>, _: u64, _: &OsStr, reply: ReplyEmpty) {
        reply.error(EPERM);
    }

    fn access(&mut self, _: &Request<'_>, ino: u64, _: i32, reply: ReplyEmpty) {
        match self.path_for(ino) {
            Ok(_) => reply.ok(),
            Err(error) => reply.error(errno(&error)),
        }
    }
}

fn attr(ino: u64, entry: &Entry) -> io::Result<FileAttr> {
    let kind = if entry.directory {
        FileType::Directory
    } else {
        FileType::RegularFile
    };
    let time = UNIX_EPOCH
        .checked_add(Duration::from_secs(entry.modified_at))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid modified time"))?;
    let created = UNIX_EPOCH
        .checked_add(Duration::from_secs(entry.created_at))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid creation time"))?;
    Ok(FileAttr {
        ino,
        size: entry.size,
        blocks: entry.size.div_ceil(512),
        atime: time,
        mtime: time,
        ctime: time,
        crtime: created,
        kind,
        perm: if entry.directory {
            0o777
        } else if entry.metadata {
            0o444
        } else {
            0o666
        },
        nlink: if entry.directory { 2 } else { 1 },
        uid: unsafe { libc::geteuid() },
        gid: unsafe { libc::getegid() },
        rdev: 0,
        blksize: 4096,
        flags: 0,
    })
}

fn errno(error: &io::Error) -> c_int {
    if let Some(raw) = error.raw_os_error() {
        return raw;
    }
    match error.kind() {
        io::ErrorKind::NotFound => ENOENT,
        io::ErrorKind::PermissionDenied => EACCES,
        io::ErrorKind::InvalidInput | io::ErrorKind::InvalidData => EINVAL,
        io::ErrorKind::NotADirectory => ENOTDIR,
        io::ErrorKind::Unsupported => ENOSYS,
        _ => EIO,
    }
}

const EISDIR: c_int = libc::EISDIR;
