//! Serialized requests from the FSKit volume to the shared storage implementation.
use crate::storage::Store;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    ffi::{CStr, CString, c_char, c_void},
    io,
    path::Path,
    sync::Mutex,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    operation: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    offset: u64,
    #[serde(default)]
    length: usize,
    #[serde(default)]
    retention: u64,
    #[serde(default)]
    data: Vec<u8>,
}

fn response(result: io::Result<Value>) -> *mut c_char {
    let value = match result {
        Ok(value) => json!({"result": value}),
        Err(error) => {
            let code = error.raw_os_error().unwrap_or(match error.kind() {
                io::ErrorKind::PermissionDenied => libc::EPERM,
                io::ErrorKind::AlreadyExists => libc::EEXIST,
                io::ErrorKind::NotFound => libc::ENOENT,
                io::ErrorKind::NotADirectory => libc::ENOTDIR,
                io::ErrorKind::WouldBlock => libc::EBUSY,
                io::ErrorKind::InvalidInput => libc::EINVAL,
                _ => libc::EIO,
            });
            json!({"error": code, "message": error.to_string()})
        }
    };
    CString::new(value.to_string()).unwrap().into_raw()
}

/// Open a storage handle. Returns null on failure and sets a JSON error response.
/// # Safety
/// `path` must be a valid null-terminated UTF-8 path; `error` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ab_worm_storage_open(
    path: *const c_char,
    error: *mut *mut c_char,
) -> *mut c_void {
    // SAFETY: the caller supplies the documented C strings and output pointer.
    let result = unsafe { CStr::from_ptr(path) }
        .to_str()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid storage path"))
        .and_then(|path| Store::open(Path::new(path)));
    match result {
        Ok(store) => Box::into_raw(Box::new(Mutex::new(store))).cast(),
        Err(problem) => {
            unsafe {
                *error = response(Err(problem));
            }
            std::ptr::null_mut()
        }
    }
}

/// Close a storage handle after all requests have returned.
/// # Safety
/// `handle` must have been returned by `ab_worm_storage_open` and be closed once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ab_worm_storage_close(handle: *mut c_void) {
    // SAFETY: ownership is transferred back exactly once by the caller.
    unsafe {
        drop(Box::from_raw(handle.cast::<Mutex<Store>>()));
    }
}

/// Execute one storage operation and return an owned JSON response.
/// # Safety
/// `handle` must be live and `request` must be a null-terminated UTF-8 JSON string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ab_worm_storage_request(
    handle: *mut c_void,
    request: *const c_char,
) -> *mut c_char {
    let execute = || -> io::Result<Value> {
        // SAFETY: both pointers obey the documented lifetime and representation.
        let text = unsafe { CStr::from_ptr(request) }.to_bytes();
        if text.len() > 8 * 1024 * 1024 {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        let request: Request = serde_json::from_slice(text)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        let mutex = unsafe { &*handle.cast::<Mutex<Store>>() };
        let mut store = mutex
            .lock()
            .map_err(|_| io::Error::other("Storage lock poisoned"))?;
        match request.operation.as_str() {
            "entry" => Ok(json!(store.entry(&request.path)?)),
            "entries" => Ok(json!(store.entries(&request.path)?)),
            "read" => Ok(json!(store.read_at(
                &request.path,
                request.offset,
                request.length
            )?)),
            "create" => {
                store.create(&request.path, request.retention)?;
                Ok(json!(store.entry(&request.path)?))
            }
            "mkdir" => {
                store.mkdir(&request.path)?;
                Ok(json!(store.entry(&request.path)?))
            }
            "write" => {
                store.append_at(&request.path, request.offset, &request.data)?;
                Ok(json!(request.data.len()))
            }
            "delete" => {
                store.remove_entry(&request.path)?;
                Ok(Value::Null)
            }
            // Every accepted mutation is already synchronized before its response.
            "sync" => Ok(Value::Null),
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
        }
    };
    match std::panic::catch_unwind(execute) {
        Ok(result) => response(result),
        Err(_) => response(Err(io::Error::other("Storage request panicked"))),
    }
}

/// Release a response string.
/// # Safety
/// `value` must be a response returned by this module and freed exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ab_worm_string_free(value: *mut c_char) {
    // SAFETY: this pointer was allocated by CString::into_raw in response().
    unsafe {
        drop(CString::from_raw(value));
    }
}
