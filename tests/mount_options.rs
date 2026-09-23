use autobricks_worm::cli::mount::MountOptions;
fn parse(args: &[&str]) -> std::io::Result<MountOptions> {
    MountOptions::parse(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>())
}
#[test]
fn retention_is_required_and_expressed_in_days() {
    let options = parse(&["mount", "source", "target", "--retain", "365"]).unwrap();
    assert_eq!(options.retention_days, 365);
    assert!(parse(&["mount", "source", "target"]).is_err());
    assert!(parse(&["mount", "source", "target", "--retain", "-1"]).is_err());
    assert!(
        parse(&[
            "mount",
            "source",
            "target",
            "--retain",
            "18446744073709551615"
        ])
        .is_err()
    );
    assert!(parse(&["mount", "source", "target", "--retain", "0"]).is_ok());
}
