//! Appendable WORM policies and filesystem integration.

#[cfg(target_os = "linux")]
pub mod fuse;
pub mod metadata;
pub mod policy;

pub use policy::{EntryPolicy, FilePolicy, PolicyError};
