//! FUSE attributes callbacks.

macro_rules! attributes_callbacks {
    () => {
        fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
            self.inner.lookup(_req, parent, name, reply)
        }

        fn getattr(&mut self, _req: &Request<'_>, ino: u64, fh: Option<u64>, reply: ReplyAttr) {
            self.inner.getattr(_req, ino, fh, reply)
        }

        fn setattr(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            mode: Option<u32>,
            uid: Option<u32>,
            gid: Option<u32>,
            size: Option<u64>,
            _atime: Option<TimeOrNow>,
            _mtime: Option<TimeOrNow>,
            _ctime: Option<SystemTime>,
            fh: Option<u64>,
            _crtime: Option<SystemTime>,
            _chgtime: Option<SystemTime>,
            _bkuptime: Option<SystemTime>,
            flags: Option<u32>,
            reply: ReplyAttr,
        ) {
            self.inner.setattr(
                _req, ino, mode, uid, gid, size, _atime, _mtime, _ctime, fh, _crtime, _chgtime,
                _bkuptime, flags, reply,
            )
        }

        fn access(&mut self, _req: &Request<'_>, ino: u64, mask: i32, reply: ReplyEmpty) {
            self.inner.access(_req, ino, mask, reply)
        }

        fn statfs(&mut self, _req: &Request<'_>, _ino: u64, reply: ReplyStatfs) {
            self.inner.statfs(_req, _ino, reply)
        }

        fn setxattr(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            name: &OsStr,
            _value: &[u8],
            flags: i32,
            position: u32,
            reply: ReplyEmpty,
        ) {
            self.inner
                .setxattr(_req, ino, name, _value, flags, position, reply)
        }

        fn getxattr(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            name: &OsStr,
            size: u32,
            reply: ReplyXattr,
        ) {
            self.inner.getxattr(_req, ino, name, size, reply)
        }

        fn listxattr(&mut self, _req: &Request<'_>, ino: u64, size: u32, reply: ReplyXattr) {
            self.inner.listxattr(_req, ino, size, reply)
        }

        fn removexattr(&mut self, _req: &Request<'_>, ino: u64, name: &OsStr, reply: ReplyEmpty) {
            self.inner.removexattr(_req, ino, name, reply)
        }
    };
}

pub(super) use attributes_callbacks;
