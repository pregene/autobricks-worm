use super::{FilePolicy, PolicyError};

/// Policy for a created filesystem entry with a fixed name and parent directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryPolicy {
    File(FilePolicy),
    Directory,
    Metadata,
}

impl EntryPolicy {
    /// Reject renaming and moving an entry, including replacement and exchange.
    pub fn check_rename(&self) -> Result<(), PolicyError> {
        Err(PolicyError::ImmutablePath)
    }
}
