//! Append, retention, and immutable path policies.

mod entry;
mod error;
mod file;

pub use entry::EntryPolicy;
pub use error::PolicyError;
pub use file::FilePolicy;
