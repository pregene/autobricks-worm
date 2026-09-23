//! macFUSE integration and macOS-specific callbacks.

pub use crate::fuse::NamespaceGuard;

macro_rules! macos_callbacks {
    () => {
        fn exchange(
            &mut self,
            _req: &Request<'_>,
            _parent: u64,
            _name: &OsStr,
            _newparent: u64,
            _newname: &OsStr,
            _options: u64,
            reply: ReplyEmpty,
        ) {
            reply.error(libc::EPERM);
        }

        fn setvolname(&mut self, _req: &Request<'_>, _name: &OsStr, reply: ReplyEmpty) {
            reply.error(libc::EPERM);
        }

        fn getxtimes(&mut self, req: &Request<'_>, ino: u64, reply: ReplyXTimes) {
            self.inner.getxtimes(req, ino, reply)
        }
    };
}

pub(crate) use macos_callbacks;
