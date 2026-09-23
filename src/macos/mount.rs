//! Register the bundled FSKit extension and invoke the system mount helper.
use crate::cli::mount::MountOptions;
use std::{fs, io, path::Path, process::Command};

fn command(program: &str, args: &[&std::ffi::OsStr]) -> io::Result<()> {
    let output = Command::new(program).args(args).output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "{program} failed: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )))
}

pub fn mount(options: MountOptions) -> io::Result<()> {
    let version = Command::new("/usr/bin/sw_vers")
        .arg("-productVersion")
        .output()?;
    let major: u32 = String::from_utf8_lossy(&version.stdout)
        .split('.')
        .next()
        .unwrap_or("")
        .trim()
        .parse()
        .unwrap_or(0);
    if major < 26 {
        return Err(io::Error::other(
            "Directory-backed FSKit mounting requires macOS 26 or later",
        ));
    }
    if !fs::symlink_metadata(&options.source)?.is_dir() {
        return Err(io::Error::other(
            "Source must be a directory, not a symbolic link",
        ));
    }
    let source = options.source.canonicalize()?;
    let executable = std::env::current_exe()?;
    let app = executable.parent().unwrap().join("Autobricks WORM.app");
    let extension = app.join("Contents/Extensions/AutobricksWORM.appex");
    if !extension.is_dir() {
        return Err(io::Error::other(
            "FSKit app bundle not found beside ab-worm. Run ./build.sh --release first",
        ));
    }
    let mountpoint = if options.mountpoint.exists() {
        options.mountpoint.canonicalize()?
    } else {
        let parent = options
            .mountpoint
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        parent.canonicalize()?.join(
            options
                .mountpoint
                .file_name()
                .ok_or_else(|| io::Error::other("Invalid mountpoint"))?,
        )
    };
    if source.starts_with(&mountpoint) || mountpoint.starts_with(&source) {
        return Err(io::Error::other(
            "Source and mountpoint must be separate, non-nested directories",
        ));
    }
    if !options.mountpoint.exists() {
        fs::create_dir(&options.mountpoint)?;
    }
    if !fs::symlink_metadata(&options.mountpoint)?.is_dir() {
        return Err(io::Error::other(
            "Mountpoint must be a directory, not a symbolic link",
        ));
    }
    if fs::read_dir(&mountpoint)?.next().is_some() {
        return Err(io::Error::other("Mountpoint must be empty"));
    }
    // Validate the backing store and release its lock before FSKit opens it.
    drop(crate::storage::Store::open(&source)?);
    command(
        "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister",
        &["-f".as_ref(), app.as_os_str()],
    )?;
    command(
        "/usr/bin/pluginkit",
        &["-a".as_ref(), extension.as_os_str()],
    )?;
    let retention = format!("retain={}", options.retention_days);
    command("/sbin/mount", &["-t".as_ref(), "abworm".as_ref(), "-o".as_ref(), retention.as_ref(), source.as_os_str(), mountpoint.as_os_str()])
        .map_err(|error| io::Error::other(format!("{error}\nEnable Autobricks WORM in System Settings > General > Login Items & Extensions > File System Extensions, then retry.")))?;
    println!(
        "Mounted {} at {} (new-file retention: {} days)",
        source.display(),
        mountpoint.display(),
        options.retention_days
    );
    Ok(())
}

pub fn unmount(path: &Path) -> io::Result<()> {
    let path = path.canonicalize()?;
    command("/sbin/umount", &[path.as_os_str()])?;
    println!("Unmounted {}", path.display());
    Ok(())
}
