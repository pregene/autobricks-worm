use crate::PolicyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataOperation {
    Read,
    GetAttributes,
    Create,
    Write,
    Truncate,
    Delete,
    Rename,
    SetAttributes,
    CreateLink,
}

/// Check operations issued by a user against a metadata entry.
/// Internal metadata updates use a separate storage path.
pub fn check_user_access(operation: MetadataOperation) -> Result<(), PolicyError> {
    match operation {
        MetadataOperation::Read | MetadataOperation::GetAttributes => Ok(()),
        _ => Err(PolicyError::ReadOnlyMetadata),
    }
}
