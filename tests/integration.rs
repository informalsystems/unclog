//! Integration tests for `unclog`.

use std::path::PathBuf;
use unclog::{Component, ComponentLoader, Project, Result};

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

#[test]
fn full() {
    env_logger::init();
    let project = Project::new_with_component_loader("./tests/full", MockLoader);
    let changelog = project.read_changelog().unwrap();
    let expected = std::fs::read_to_string("./tests/full/expected.md").unwrap();
    assert_eq!(expected, changelog.to_string());
}
