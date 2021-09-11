//! Integration tests for `unclog`.

use lazy_static::lazy_static;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use unclog::{Changelog, Component, ComponentLoader, Config, PlatformId, Project, Result};

struct MockLoader;

impl ComponentLoader for MockLoader {
    fn get_component(&mut self, name: &str) -> Result<Option<Component>> {
        match name {
            "component2" => Ok(Some(Component {
                name: "component2".to_owned(),
                rel_path: PathBuf::from("2nd-component"),
            })),
            _ => Ok(None),
        }
    }
}

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
    init_logger();
    let project = Project::new_with_component_loader("./tests/full", MockLoader);
    let config = Config::default();
    let changelog = project.read_changelog(&config).unwrap();
    let expected = std::fs::read_to_string("./tests/full/expected.md").unwrap();
    assert_eq!(expected, changelog.render(&config));
}

#[test]
fn change_template_rendering() {
    init_logger();
    let config = Config::read_from_file("./tests/full/config.toml").unwrap();
    let rendered_change = Changelog::render_unreleased_entry_from_template(
        &config,
        Path::new("./tests/full"),
        "breaking-changes",
        None,
        "some-new-breaking-change",
        PlatformId::Issue(123),
        "This introduces a new *breaking* change",
    )
    .unwrap();
    assert_eq!(rendered_change, "- This introduces a new *breaking* change ([#123](https://github.com/org/project/issues/123))")
}
