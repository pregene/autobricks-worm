use std::process::ExitCode;

fn main() -> ExitCode {
    println!(
        "Autobricks WORM Filesystem {} (C) 2026 Autobricks, Co.",
        env!("AB_WORM_VERSION")
    );
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => print_help(),
        [arg] if arg == "--help" || arg == "-h" => print_help(),
        [arg] if arg == "--version" || arg == "-V" => {}
        _ => {
            eprintln!("Unsupported arguments. Run ab-worm --help.");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

fn print_help() {
    println!(
        "Usage: ab-worm [--help | --version]\n\
         \nOptions:\n\
         \x20 -h, --help       Show usage\n\
         \x20 -V, --version    Show product version"
    );
}
