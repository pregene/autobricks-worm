//! Apple FSKit integration.

pub mod bridge;

pub const NAME: &str = "macos";
pub const EXECUTABLE_NAME: &str = "ab-worm";

pub mod mount;
pub mod storage_bridge;
