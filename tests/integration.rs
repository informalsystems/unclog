//! Integration tests for `unclog`.

use lazy_static::lazy_static;
use std::{path::Path, sync::Mutex};
use unclog::{Changelog, Config, PlatformId};

lazy_static! {
    static ref LOGGING_INITIALIZED: Mutex<u8> = Mutex::new(0);
}

fn init_logger() {
    let mut initialized = LOGGING_INITIALIZED.lock().unwrap();
    if *initialized == 0 {
        env_logger::init();
        *initialized = 1;
        log::debug!("env logger initialized");
    } else {
        log::debug!("env logger already initialized");
    }
}

#[test]
fn full() {
    const CONFIG_FILE: &str = r#"
[components.all]
component2 = { name = "component2", path = "2nd-component" }
"#;

    init_logger();
    let config = toml::from_str(CONFIG_FILE).unwrap();
    let changelog = Changelog::read_from_dir(&config, "./tests/full").unwrap();
    let expected = std::fs::read_to_string("./tests/full/expected.md").unwrap();
    assert_eq!(expected, changelog.render(&config));
}

#[test]
fn change_template_rendering() {
    init_logger();
    let config = Config::read_from_file("./tests/full/config.toml").unwrap();
    let cases = vec![
        (PlatformId::Issue(123), "- This introduces a new *breaking* change\n  ([#123](https://github.com/org/project/issues/123))"),
        (PlatformId::PullRequest(23), "- This introduces a new *breaking* change\n  ([#23](https://github.com/org/project/pull/23))"),
    ];
    for (platform_id, expected) in cases {
        let actual = Changelog::render_unreleased_entry_from_template(
            &config,
            Path::new("./tests/full"),
            "breaking-changes",
            None,
            "some-new-breaking-change",
            platform_id,
            "This introduces a new *breaking* change",
        )
        .unwrap();
        assert_eq!(actual, expected);
    }
}
