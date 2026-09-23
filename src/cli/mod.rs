//! Direct backing-store commands for development and verification.
mod storage;

pub use storage::run;

pub fn help(executable: &str) {
    println!(
        "Usage: {executable} [--help | --version]\n\
        \x20      {executable} storage ROOT create PATH RETENTION_SECONDS\n\
        \x20      {executable} storage ROOT append PATH < INPUT\n\
        \x20      {executable} storage ROOT read PATH\n\
        \x20      {executable} storage ROOT meta PATH\n\
        \x20      {executable} storage ROOT verify PATH\n\
        \x20      {executable} storage ROOT delete PATH\n\
        \x20      {executable} storage ROOT mkdir PATH\n\n\
        ROOT is a private backing directory. PATH is relative to ROOT.\n\
        Retention is fixed at file creation. Each accepted write updates checksum and LOCK."
    );
}
