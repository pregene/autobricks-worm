//! Appendable WORM policies and filesystem integration.

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub use linux as platform;
#[cfg(target_os = "macos")]
pub use macos as platform;
#[cfg(target_os = "windows")]
pub use windows as platform;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
compile_error!("ab-worm supports Linux, macOS, and Windows build targets");

pub mod metadata;
pub mod policy;

pub use policy::{EntryPolicy, FilePolicy, PolicyError};
