//! FUSE namespace callbacks.

use std::ffi::OsStr;

use crate::{PolicyError, metadata};

pub(super) fn check_name(name: &OsStr) -> Result<(), libc::c_int> {
    metadata::check_user_entry_name(name).map_err(|error| match error {
        PolicyError::InvalidName => libc::EINVAL,
        _ => libc::EPERM,
    })
}

macro_rules! namespace_callbacks {
    () => {
        fn create(
            &mut self,
            _req: &Request<'_>,
            parent: u64,
            name: &OsStr,
            mode: u32,
            umask: u32,
            flags: i32,
            reply: ReplyCreate,
        ) {
            if let Err(error) = $crate::fuse::namespace::check_name(name) {
                reply.error(error);
                return;
            }
            self.inner
                .create(_req, parent, name, mode, umask, flags, reply)
        }

        fn mknod(
            &mut self,
            _req: &Request<'_>,
            parent: u64,
            name: &OsStr,
            mode: u32,
            umask: u32,
            rdev: u32,
            reply: ReplyEntry,
        ) {
            if let Err(error) = $crate::fuse::namespace::check_name(name) {
                reply.error(error);
                return;
            }
            self.inner
                .mknod(_req, parent, name, mode, umask, rdev, reply)
        }

        fn mkdir(
            &mut self,
            _req: &Request<'_>,
            parent: u64,
            name: &OsStr,
            mode: u32,
            umask: u32,
            reply: ReplyEntry,
        ) {
            if let Err(error) = $crate::fuse::namespace::check_name(name) {
                reply.error(error);
                return;
            }
            self.inner.mkdir(_req, parent, name, mode, umask, reply)
        }

        fn symlink(
            &mut self,
            _req: &Request<'_>,
            parent: u64,
            link_name: &OsStr,
            target: &Path,
            reply: ReplyEntry,
        ) {
            if let Err(error) = $crate::fuse::namespace::check_name(link_name) {
                reply.error(error);
                return;
            }
            self.inner.symlink(_req, parent, link_name, target, reply)
        }

        fn link(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            newparent: u64,
            newname: &OsStr,
            reply: ReplyEntry,
        ) {
            if let Err(error) = $crate::fuse::namespace::check_name(newname) {
                reply.error(error);
                return;
            }
            self.inner.link(_req, ino, newparent, newname, reply)
        }

        fn rename(
            &mut self,
            _req: &Request<'_>,
            parent: u64,
            name: &OsStr,
            newparent: u64,
            newname: &OsStr,
            flags: u32,
            reply: ReplyEmpty,
        ) {
            let _ = (_req, parent, name, newparent, newname, flags);
            reply.error(EPERM);
        }

        fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
            self.inner.unlink(_req, parent, name, reply)
        }

        fn rmdir(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
            self.inner.rmdir(_req, parent, name, reply)
        }
    };
}

pub(super) use namespace_callbacks;
