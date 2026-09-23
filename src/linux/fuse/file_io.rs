//! FUSE file io callbacks.

macro_rules! file_io_callbacks {
    () => {
        fn open(&mut self, _req: &Request<'_>, _ino: u64, _flags: i32, reply: ReplyOpen) {
            self.inner.open(_req, _ino, _flags, reply)
        }

        fn read(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            offset: i64,
            size: u32,
            flags: i32,
            lock_owner: Option<u64>,
            reply: ReplyData,
        ) {
            self.inner
                .read(_req, ino, fh, offset, size, flags, lock_owner, reply)
        }

        fn write(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            offset: i64,
            data: &[u8],
            write_flags: u32,
            flags: i32,
            lock_owner: Option<u64>,
            reply: ReplyWrite,
        ) {
            self.inner.write(
                _req,
                ino,
                fh,
                offset,
                data,
                write_flags,
                flags,
                lock_owner,
                reply,
            )
        }

        fn flush(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            lock_owner: u64,
            reply: ReplyEmpty,
        ) {
            self.inner.flush(_req, ino, fh, lock_owner, reply)
        }

        fn release(
            &mut self,
            _req: &Request<'_>,
            _ino: u64,
            _fh: u64,
            _flags: i32,
            _lock_owner: Option<u64>,
            _flush: bool,
            reply: ReplyEmpty,
        ) {
            self.inner
                .release(_req, _ino, _fh, _flags, _lock_owner, _flush, reply)
        }

        fn fsync(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            datasync: bool,
            reply: ReplyEmpty,
        ) {
            self.inner.fsync(_req, ino, fh, datasync, reply)
        }

        fn readlink(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyData) {
            self.inner.readlink(_req, ino, reply)
        }
    };
}

pub(super) use file_io_callbacks;
