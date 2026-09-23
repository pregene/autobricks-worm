//! Metadata naming and read-only user access policies.

mod access;
mod names;

pub use access::{MetadataOperation, check_user_access};
pub use names::{check_user_entry_name, name_for};
