//! Linux build configuration and filesystem integration.

pub mod fuse;
pub mod mount;

pub const NAME: &str = "linux";
pub const EXECUTABLE_NAME: &str = "ab-worm";
