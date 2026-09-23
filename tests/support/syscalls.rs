use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub fn fopen(path: &Path) -> std::io::Result<()> {
    let path = CString::new(path.as_os_str().as_bytes()).unwrap();
    // SAFETY: Both strings are NUL-terminated and remain alive during fopen.
    let file = unsafe { libc::fopen(path.as_ptr(), c"w".as_ptr()) };
    if file.is_null() {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: file is a live FILE pointer returned by fopen, closed once here.
    if unsafe { libc::fclose(file) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

pub fn mkfifo(path: &Path) -> std::io::Result<()> {
    let path = CString::new(path.as_os_str().as_bytes()).unwrap();
    // SAFETY: path is a valid NUL-terminated string for the duration of the call.
    if unsafe { libc::mkfifo(path.as_ptr(), 0o600) } == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
