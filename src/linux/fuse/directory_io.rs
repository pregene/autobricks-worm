//! FUSE directory io callbacks.

macro_rules! directory_io_callbacks {
    () => {
        fn opendir(&mut self, _req: &Request<'_>, _ino: u64, _flags: i32, reply: ReplyOpen) {
            self.inner.opendir(_req, _ino, _flags, reply)
        }

        fn readdir(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            offset: i64,
            reply: ReplyDirectory,
        ) {
            self.inner.readdir(_req, ino, fh, offset, reply)
        }

        fn readdirplus(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            offset: i64,
            reply: ReplyDirectoryPlus,
        ) {
            self.inner.readdirplus(_req, ino, fh, offset, reply)
        }

        fn releasedir(
            &mut self,
            _req: &Request<'_>,
            _ino: u64,
            _fh: u64,
            _flags: i32,
            reply: ReplyEmpty,
        ) {
            self.inner.releasedir(_req, _ino, _fh, _flags, reply)
        }

        fn fsyncdir(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            datasync: bool,
            reply: ReplyEmpty,
        ) {
            self.inner.fsyncdir(_req, ino, fh, datasync, reply)
        }
    };
}

pub(super) use directory_io_callbacks;
