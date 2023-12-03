//! Integration tests for `unclog`.

use lazy_static::lazy_static;
use std::{path::Path, sync::Mutex};
use unclog::{ChangeSetComponentPath, Changelog, Config, EntryReleasePath, PlatformId};

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
component1 = { name = "component1" }
component2 = { name = "Component 2", path = "2nd-component" }
"#;

    init_logger();
    let config = toml::from_str(CONFIG_FILE).unwrap();
    let changelog = Changelog::read_from_dir(&config, "./tests/full").unwrap();
    let expected = std::fs::read_to_string("./tests/full/expected.md").unwrap();
    assert_eq!(expected, changelog.render_all(&config));
}

#[test]
fn released_only() {
    const CONFIG_FILE: &str = r#"
[components.all]
component1 = { name = "component1" }
component2 = { name = "Component 2", path = "2nd-component" }
"#;

    init_logger();
    let config = toml::from_str(CONFIG_FILE).unwrap();
    let changelog = Changelog::read_from_dir(&config, "./tests/full").unwrap();
    let expected = std::fs::read_to_string("./tests/full/expected-released-only.md").unwrap();
    assert_eq!(expected, changelog.render_released(&config));
}

#[test]
fn full_sorted_by_entry_text() {
    const CONFIG_FILE: &str = r#"
[change_set_sections]
sort_entries_by = "entry-text"

[components.all]
component1 = { name = "component1" }
component2 = { name = "Component 2", path = "2nd-component" }
"#;

    init_logger();
    let config = toml::from_str(CONFIG_FILE).unwrap();
    let changelog = Changelog::read_from_dir(&config, "./tests/full").unwrap();
    let expected =
        std::fs::read_to_string("./tests/full/expected-sorted-by-entry-text.md").unwrap();
    assert_eq!(expected, changelog.render_all(&config));
}

#[test]
fn full_sorted_by_release_date() {
    const CONFIG_FILE: &str = r#"
sort_releases_by = ["date"]
release_date_formats = [
    "*%d %b %Y*"
]

[components.all]
component1 = { name = "component1" }
component2 = { name = "Component 2", path = "2nd-component" }
"#;

    init_logger();
    let config = toml::from_str(CONFIG_FILE).unwrap();
    let changelog = Changelog::read_from_dir(&config, "./tests/full").unwrap();
    let expected =
        std::fs::read_to_string("./tests/full/expected-sorted-by-release-date.md").unwrap();
    assert_eq!(expected, changelog.render_all(&config));
}

#[test]
fn change_template_rendering() {
    init_logger();
    let config = Config::read_from_file("./tests/full/config.toml").unwrap();
    let cases = vec![
        (PlatformId::Issue(123), "- This introduces a new *breaking* change\n  ([\\#123](https://github.com/org/project/issues/123))"),
        (PlatformId::PullRequest(23), "- This introduces a new *breaking* change\n  ([\\#23](https://github.com/org/project/pull/23))"),
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

#[test]
fn entry_iteration() {
    const CONFIG_FILE: &str = r#"
[components.all]
component1 = { name = "component1" }
component2 = { name = "Component 2", path = "2nd-component" }
"#;

    const UNRELEASED: &str = "Unreleased";
    const GENERAL: &str = "General";
    const EXPECTED_ENTRIES: &[(&str, &str, &str, &str)] = &[
        (
            UNRELEASED,
            "FEATURES",
            GENERAL,
            "- Travel through space as a beneficial example",
        ),
        (UNRELEASED, "IMPROVEMENTS", GENERAL, "- Eat the profile"),
        (
            "v0.2.1",
            "BREAKING CHANGES",
            "Component 2",
            "- Gargle the truffle",
        ),
        (
            "v0.2.1",
            "BREAKING CHANGES",
            "Component 2",
            "- Travel the gravel",
        ),
        (
            "v0.2.1",
            "BREAKING CHANGES",
            "Component 2",
            "- Laugh at the gaggle",
        ),
        ("v0.2.1", "FEATURES", GENERAL, "- Nibble the bubbles"),
        ("v0.2.1", "FEATURES", GENERAL, "- Carry the wobbles"),
        ("v0.2.1", "FEATURES", "component1", "- Fasten the handles"),
        ("v0.2.1", "FEATURES", "component1", "- Hasten the sandals"),
    ];

    init_logger();
    let config = toml::from_str(CONFIG_FILE).unwrap();
    let changelog = Changelog::read_from_dir(&config, "./tests/full").unwrap();
    let mut entries = changelog.entries();

    for (i, (expected_release, expected_section, expected_component, expected_entry_details)) in
        EXPECTED_ENTRIES.iter().enumerate()
    {
        let next = entries.next().unwrap();
        assert_eq!(next.changelog, &changelog);
        let change_set_path = match next.release_path {
            EntryReleasePath::Unreleased(change_set_path) => {
                assert_eq!(&UNRELEASED, expected_release, "for entry {i}");
                change_set_path
            }
            EntryReleasePath::Released(release, change_set_path) => {
                assert_eq!(&release.id, expected_release, "for entry {i}");
                assert_eq!(
                    &change_set_path.section_path.change_set_section.title,
                    expected_section
                );
                change_set_path
            }
        };
        assert_eq!(
            &change_set_path.section_path.change_set_section.title,
            expected_section
        );
        match change_set_path.section_path.component_path {
            ChangeSetComponentPath::General(entry) => {
                assert_eq!(&GENERAL, expected_component);
                assert_eq!(&entry.details, expected_entry_details);
            }
            ChangeSetComponentPath::Component(component_section, entry) => {
                assert_eq!(&component_section.name, expected_component);
                assert_eq!(&entry.details, expected_entry_details);
            }
        }
    }
}
