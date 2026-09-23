use autobricks_worm::{cli, platform::EXECUTABLE_NAME};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let banner = format!(
        "Autobricks WORM Filesystem {} (C) 2026 Autobricks, Co.",
        env!("AB_WORM_VERSION")
    );
    if args.first().is_some_and(|arg| arg == "storage") {
        eprintln!("{banner}");
        if let Err(error) = cli::run(&args) {
            eprintln!("Storage operation failed: {error}");
            return ExitCode::FAILURE;
        }
    } else if args
        .first()
        .is_some_and(|arg| arg == "mount" || arg == "unmount")
    {
        eprintln!("{banner}");
        let result = if args.first().is_some_and(|arg| arg == "mount") {
            cli::mount::run(&args)
        } else {
            cli::mount::unmount(&args)
        };
        if let Err(error) = result {
            eprintln!("Mount operation failed: {error}");
            return ExitCode::FAILURE;
        }
    } else {
        println!("{banner}");
        match args.as_slice() {
            [] => cli::help(EXECUTABLE_NAME),
            [arg] if arg == "--help" || arg == "-h" => cli::help(EXECUTABLE_NAME),
            [arg] if arg == "--version" || arg == "-V" => {}
            _ => {
                eprintln!("Unsupported arguments. Run {EXECUTABLE_NAME} --help.");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
