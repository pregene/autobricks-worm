use std::io;

fn main() -> io::Result<()> {
    println!("cargo::rerun-if-changed=VERSION");
    println!("cargo::rerun-if-env-changed=AB_WORM_BUILD_VERSION");
    let version =
        std::env::var("AB_WORM_BUILD_VERSION").or_else(|_| std::fs::read_to_string("VERSION"))?;
    let version = version.trim();
    let parts: Vec<_> = version.split('.').collect();
    if parts.len() != 3
        || !parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|b| b.is_ascii_digit())
                && (part.len() == 1 || !part.starts_with('0'))
                && part.parse::<u64>().is_ok()
        })
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VERSION must contain major.minor.patch as unsigned integers",
        ));
    }
    println!("cargo::rustc-env=AB_WORM_VERSION={version}");
    Ok(())
}
