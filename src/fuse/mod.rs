//! FUSE namespace enforcement for reserved metadata names.
//!
//! Filesystem callback signatures follow fuser 0.16.0 (MIT).
//! See licenses/fuser-0.16.0-MIT.txt for the upstream notice.

use std::ffi::OsStr;
use std::path::Path;
use std::time::SystemTime;

use fuser::*;
use libc::{EPERM, c_int};

mod attributes;
mod directory_io;
mod file_control;
mod file_io;
mod lifecycle;
mod namespace;

/// Enforce reserved names before dispatching creation to the backing filesystem.
/// Rename operations are rejected for all existing entries.
pub struct NamespaceGuard<F> {
    inner: F,
}

impl<F: Filesystem> NamespaceGuard<F> {
    pub fn new(inner: F) -> Self {
        Self { inner }
    }

    /// Mount the guarded filesystem and process requests until unmounted.
    pub fn mount(self, mountpoint: &Path, options: &[MountOption]) -> std::io::Result<()> {
        fuser::mount2(self, mountpoint, options)
    }
}

impl<F: Filesystem> Filesystem for NamespaceGuard<F> {
    #[cfg(target_os = "macos")]
    crate::macos::fuse::macos_callbacks!();
    lifecycle::lifecycle_callbacks!();
    namespace::namespace_callbacks!();
    attributes::attributes_callbacks!();
    file_io::file_io_callbacks!();
    directory_io::directory_io_callbacks!();
    file_control::file_control_callbacks!();
}
