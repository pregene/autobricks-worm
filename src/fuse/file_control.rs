//! FUSE file control callbacks.

macro_rules! file_control_callbacks {
    () => {
        fn getlk(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            lock_owner: u64,
            start: u64,
            end: u64,
            typ: i32,
            pid: u32,
            reply: ReplyLock,
        ) {
            self.inner
                .getlk(_req, ino, fh, lock_owner, start, end, typ, pid, reply)
        }

        fn setlk(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            lock_owner: u64,
            start: u64,
            end: u64,
            typ: i32,
            pid: u32,
            sleep: bool,
            reply: ReplyEmpty,
        ) {
            self.inner.setlk(
                _req, ino, fh, lock_owner, start, end, typ, pid, sleep, reply,
            )
        }

        fn bmap(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            blocksize: u32,
            idx: u64,
            reply: ReplyBmap,
        ) {
            self.inner.bmap(_req, ino, blocksize, idx, reply)
        }

        fn ioctl(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            flags: u32,
            cmd: u32,
            in_data: &[u8],
            out_size: u32,
            reply: ReplyIoctl,
        ) {
            self.inner
                .ioctl(_req, ino, fh, flags, cmd, in_data, out_size, reply)
        }

        fn poll(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            ph: PollHandle,
            events: u32,
            flags: u32,
            reply: ReplyPoll,
        ) {
            self.inner.poll(_req, ino, fh, ph, events, flags, reply)
        }

        fn fallocate(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            offset: i64,
            length: i64,
            mode: i32,
            reply: ReplyEmpty,
        ) {
            self.inner
                .fallocate(_req, ino, fh, offset, length, mode, reply)
        }

        fn lseek(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            offset: i64,
            whence: i32,
            reply: ReplyLseek,
        ) {
            self.inner.lseek(_req, ino, fh, offset, whence, reply)
        }

        fn copy_file_range(
            &mut self,
            _req: &Request<'_>,
            ino_in: u64,
            fh_in: u64,
            offset_in: i64,
            ino_out: u64,
            fh_out: u64,
            offset_out: i64,
            len: u64,
            flags: u32,
            reply: ReplyWrite,
        ) {
            self.inner.copy_file_range(
                _req, ino_in, fh_in, offset_in, ino_out, fh_out, offset_out, len, flags, reply,
            )
        }
    };
}

pub(super) use file_control_callbacks;
