//! Integration tests for `unclog`.

use std::convert::TryFrom;
use unclog::{Changelog, ChangelogInput};

#[test]
fn full() {
    env_logger::init();
    let input = ChangelogInput::load_from_dir("./tests/full").unwrap();
    let changelog = Changelog::try_from(input).unwrap();
    let expected = std::fs::read_to_string("./tests/full/expected.md").unwrap();
    assert_eq!(expected, changelog.to_string());
}
