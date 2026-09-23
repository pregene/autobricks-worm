use std::{io, path::PathBuf};

#[derive(Debug)]
pub struct MountOptions {
    pub source: PathBuf,
    pub mountpoint: PathBuf,
    pub retention_days: u64,
    pub allow_other: bool,
}

impl MountOptions {
    pub fn parse(args: &[String]) -> io::Result<Self> {
        let [command, source, mountpoint, flag, days, rest @ ..] = args else {
            return Err(usage());
        };
        if command != "mount" || flag != "--retain" {
            return Err(usage());
        }
        let allow_other = match rest {
            [] => false,
            [option] if option == "--allow-other" => true,
            _ => return Err(usage()),
        };
        let retention_days: u64 = days.parse().map_err(|_| usage())?;
        retention_days.checked_mul(86400).ok_or_else(usage)?;
        Ok(Self {
            source: source.into(),
            mountpoint: mountpoint.into(),
            retention_days,
            allow_other,
        })
    }
}
fn usage() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "Usage: ab-worm mount SOURCE MOUNTPOINT --retain DAYS [--allow-other] (unsigned days, 0 permits immediate deletion)",
    )
}

pub fn run(args: &[String]) -> io::Result<()> {
    let options = MountOptions::parse(args)?;
    #[cfg(target_os = "macos")]
    {
        crate::macos::mount::mount(options)
    }
    #[cfg(target_os = "linux")]
    {
        crate::linux::mount::mount(options)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = options;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "This platform does not support mounting yet",
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
    #[cfg(target_os = "linux")]
    {
        crate::linux::mount::unmount(path.as_ref())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = path;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "This platform does not support unmounting yet",
        ))
    }
}
