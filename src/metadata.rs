//! Naming and user access policy for per-file metadata entries.

use std::ffi::{OsStr, OsString};

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

/// Reserve metadata names before creating a user file or directory.
pub fn check_user_entry_name(name: &OsStr) -> Result<(), PolicyError> {
    let bytes = name.as_encoded_bytes();
    if bytes.is_empty()
        || bytes == b"."
        || bytes == b".."
        || bytes.contains(&b'/')
        || bytes.contains(&0)
    {
        return Err(PolicyError::InvalidName);
    }
    if bytes.len() >= 5 && bytes[bytes.len() - 5..].eq_ignore_ascii_case(b".meta") {
        return Err(PolicyError::ReservedName);
    }
    Ok(())
}

/// Derive the sibling metadata name, preserving the complete data filename.
pub fn name_for(data_name: &OsStr) -> Result<OsString, PolicyError> {
    check_user_entry_name(data_name)?;
    let mut name = data_name.to_os_string();
    name.push(".meta");
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntryPolicy;

    #[test]
    fn metadata_is_readable_but_all_user_mutations_are_denied() {
        for operation in [MetadataOperation::Read, MetadataOperation::GetAttributes] {
            assert_eq!(check_user_access(operation), Ok(()));
        }
        for operation in [
            MetadataOperation::Create,
            MetadataOperation::Write,
            MetadataOperation::Truncate,
            MetadataOperation::Delete,
            MetadataOperation::Rename,
            MetadataOperation::SetAttributes,
            MetadataOperation::CreateLink,
        ] {
            assert_eq!(
                check_user_access(operation),
                Err(PolicyError::ReadOnlyMetadata)
            );
        }
        assert_eq!(
            EntryPolicy::Metadata.check_rename(),
            Err(PolicyError::ImmutablePath)
        );
    }

    #[test]
    fn metadata_names_preserve_extensions_and_reserve_the_namespace() {
        for name in ["audit.log", "audit", ".audit"] {
            let metadata = name_for(OsStr::new(name)).unwrap();
            assert_eq!(metadata, OsString::from(format!("{name}.meta")));
            assert_eq!(
                check_user_entry_name(&metadata),
                Err(PolicyError::ReservedName)
            );
        }
        assert_eq!(
            name_for(OsStr::new("audit.META")),
            Err(PolicyError::ReservedName)
        );
    }

    #[test]
    fn metadata_names_reject_paths_and_invalid_components() {
        for name in [
            "",
            ".",
            "..",
            "../audit",
            "/audit",
            "dir/audit",
            "audit\0log",
        ] {
            assert_eq!(name_for(OsStr::new(name)), Err(PolicyError::InvalidName));
        }
    }
}
