//! FUSE lifecycle callbacks.

macro_rules! lifecycle_callbacks {
    () => {
        fn init(&mut self, _req: &Request<'_>, _config: &mut KernelConfig) -> Result<(), c_int> {
            self.inner.init(_req, _config)
        }

        fn destroy(&mut self) {
            self.inner.destroy()
        }

        fn forget(&mut self, _req: &Request<'_>, _ino: u64, _nlookup: u64) {
            self.inner.forget(_req, _ino, _nlookup)
        }

        fn batch_forget(&mut self, req: &Request<'_>, nodes: &[fuse_forget_one]) {
            self.inner.batch_forget(req, nodes)
        }
    };
}

pub(super) use lifecycle_callbacks;
