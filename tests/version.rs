#[test]
fn version_constant_matches_pkg_version() {
    assert_eq!(crypto_scanner_agent::VERSION, env!("CARGO_PKG_VERSION"));
}
