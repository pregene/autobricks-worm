use std::ffi::{OsStr, OsString};

use crate::PolicyError;

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
