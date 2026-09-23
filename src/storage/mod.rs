//! Private backing storage with serialized append transactions and SHA-256 checkpoints.
mod directories;
mod durable;
mod files;
mod paths;
mod record;
mod store;
mod transaction;

pub use record::Record;
use std::{
    io,
    time::{SystemTime, UNIX_EPOCH},
};
pub use store::Store;

pub(crate) fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
pub(crate) fn denied(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message)
}
fn now() -> io::Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_secs())
}
