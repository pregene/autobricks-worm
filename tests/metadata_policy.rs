use autobricks_worm::metadata::*;
use autobricks_worm::{EntryPolicy, PolicyError};
use std::ffi::{OsStr, OsString};

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
