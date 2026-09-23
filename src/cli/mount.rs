use std::{io, path::PathBuf};

#[derive(Debug)]
pub struct MountOptions {
    pub source: PathBuf,
    pub mountpoint: PathBuf,
    pub retention_days: u64,
}

impl MountOptions {
    pub fn parse(args: &[String]) -> io::Result<Self> {
        let [command, source, mountpoint, flag, days] = args else {
            return Err(usage());
        };
        if command != "mount" || flag != "--retain" {
            return Err(usage());
        }
        let retention_days: u64 = days.parse().map_err(|_| usage())?;
        let seconds = retention_days.checked_mul(86400).ok_or_else(usage)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_secs();
        now.checked_add(seconds).ok_or_else(usage)?;
        Ok(Self {
            source: source.into(),
            mountpoint: mountpoint.into(),
            retention_days,
        })
    }
}
fn usage() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "Usage: ab-worm mount SOURCE MOUNTPOINT --retain DAYS (unsigned days, 0 permits immediate deletion)",
    )
}

pub fn run(args: &[String]) -> io::Result<()> {
    let options = MountOptions::parse(args)?;
    #[cfg(target_os = "macos")]
    {
        crate::macos::mount::mount(options)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = options;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "This mount command uses the macOS FSKit extension",
        ))
    }
}

pub fn unmount(args: &[String]) -> io::Result<()> {
    let [_, path] = args else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Usage: ab-worm unmount MOUNTPOINT",
        ));
    };
    #[cfg(target_os = "macos")]
    {
        crate::macos::mount::unmount(path.as_ref())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "This unmount command uses macOS",
        ))
    }
}
