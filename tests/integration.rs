//! Integration tests for `unclog`.

use unclog::Changelog;

#[test]
fn full() {
    env_logger::init();
    let changelog = Changelog::read_from_dir("./tests/full").unwrap();
    let expected = std::fs::read_to_string("./tests/full/expected.md").unwrap();
    assert_eq!(expected, changelog.to_string());
}
