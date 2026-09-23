//! macOS build configuration.

#[cfg(feature = "macos-fuse")]
pub mod fuse;

pub const NAME: &str = "macos";
pub const EXECUTABLE_NAME: &str = "ab-worm";
