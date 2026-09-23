use autobricks_worm::platform;

#[test]
fn build_selects_the_target_os_configuration() {
    assert_eq!(platform::NAME, std::env::consts::OS);
    assert_eq!(
        platform::EXECUTABLE_NAME,
        format!("ab-worm{}", std::env::consts::EXE_SUFFIX)
    );
}
