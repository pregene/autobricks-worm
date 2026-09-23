//! C ABI used by the native FSKit adapter.

use crate::{PolicyError, metadata};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

/// Check a filename without changing its byte encoding.
///
/// # Safety
/// `name` must point to `length` readable bytes for this call when length is nonzero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ab_worm_check_entry_name(name: *const u8, length: usize) -> i32 {
    if length == 0 || name.is_null() {
        return libc::EINVAL;
    }
    // SAFETY: the caller supplies a readable filename buffer for this call.
    let bytes = unsafe { std::slice::from_raw_parts(name, length) };
    match metadata::check_user_entry_name(OsStr::from_bytes(bytes)) {
        Ok(()) => 0,
        Err(PolicyError::ReservedName) => libc::EPERM,
        Err(_) => libc::EINVAL,
    }
}
