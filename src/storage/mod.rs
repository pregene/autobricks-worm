//! Private backing storage with serialized append transactions and SHA-256 checkpoints.
mod directories;
mod durable;
mod files;
mod paths;
mod record;
mod store;
mod transaction;
mod view;

pub use record::Record;
use std::io;
pub use store::Store;
pub use view::Entry;

pub(crate) fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
pub(crate) fn denied(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message)
}
#[cfg(target_os = "linux")]
fn now() -> io::Result<u64> {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // CLOCK_BOOTTIME is monotonic and includes suspend time. It is not affected
    // by wall-clock changes from date, NTP steps, or settimeofday.
    let result = unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut time) };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    u64::try_from(time.tv_sec).map_err(|_| invalid("Monotonic clock returned a negative time"))
}

#[cfg(not(target_os = "linux"))]
fn now() -> io::Result<u64> {
    use std::time::{SystemTime, UNIX_EPOCH};
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_secs())
}
