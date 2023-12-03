use crate::changelog::change_set_section::indent_entries;
use crate::changelog::entry::read_entries_sorted;
use crate::changelog::fs_utils::{entry_filter, path_to_str, read_and_filter_dir};
use crate::{Config, Entry, Error, Result};
use log::{debug, warn};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

/// A section of entries related to a specific component/submodule/package.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentSection {
    /// The ID of the component.
    pub id: String,
    /// The name of the component.
    pub name: String,
    /// The path to the component, from the root of the project, if any.
    /// Pre-computed and ready to render.
    pub maybe_path: Option<String>,
    /// The entries associated with the component.
    pub entries: Vec<Entry>,
}

impl ComponentSection {
    /// Returns whether or not this section is empty (it's considered empty
    /// when it has no entries).
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Attempt to load this component section from the given directory.
    pub fn read_from_dir<P>(config: &Config, path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let id = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| Error::CannotObtainName(path_to_str(path)))?
            .to_owned();
        debug!("Looking up component with ID: {}", id);
        let component = config
            .components
            .all
            .get(&id)
            .ok_or_else(|| Error::ComponentNotDefined(id.clone()))?;
        let name = component.name.clone();
        let maybe_component_path = component.maybe_path.as_ref().map(path_to_str);
        match &maybe_component_path {
            Some(component_path) => debug!(
                "Found component \"{}\" with name \"{}\" in: {}",
                id, name, component_path
            ),
            None => warn!("No path for component \"{}\"", id),
        }
        let entry_files = read_and_filter_dir(path, |e| entry_filter(config, e))?;
        let entries = read_entries_sorted(entry_files, config)?;
        Ok(Self {
            id,
            name,
            maybe_path: maybe_component_path,
            entries,
        })
    }

    pub fn render(&self, config: &Config) -> String {
        let entries_lines = indent_entries(
            &self.entries,
            config.components.entry_indent,
            config.components.entry_indent + 2,
        );
        let name = match &self.maybe_path {
            // Render as a Markdown hyperlink
            Some(path) => format!("[{}]({})", self.name, path),
            None => self.name.clone(),
        };
        let mut lines = vec![format!("{} {}", config.bullet_style, name)];
        lines.extend(entries_lines);
        lines.join("\n")
    }
}

#[derive(Debug, Clone)]
pub struct ComponentSectionIter<'a> {
    section: &'a ComponentSection,
    entry_id: usize,
}

impl<'a> ComponentSectionIter<'a> {
    pub(crate) fn new(section: &'a ComponentSection) -> Option<Self> {
        if section.is_empty() {
            None
        } else {
            Some(Self {
                section,
                entry_id: 0,
            })
        }
    }
}

impl<'a> Iterator for ComponentSectionIter<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.section.entries.get(self.entry_id)?;
        self.entry_id += 1;
        Some(entry)
    }
}

pub(crate) fn package_section_filter(entry: fs::DirEntry) -> Option<Result<PathBuf>> {
    let meta = match entry.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(Error::Io(entry.path(), e))),
    };
    if meta.is_dir() {
        Some(Ok(entry.path()))
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::{ComponentSection, Config};
    use crate::Entry;

    const RENDERED_WITH_PATH: &str = r#"- [Some project](./some-project/)
  - Issue 1
  - Issue 2
  - Issue 3"#;

    const RENDERED_WITHOUT_PATH: &str = r#"- some-project
  - Issue 1
  - Issue 2
  - Issue 3"#;

    #[test]
    fn with_path() {
        let ps = ComponentSection {
            id: "some-project".to_owned(),
            name: "Some project".to_owned(),
            maybe_path: Some("./some-project/".to_owned()),
            entries: test_entries(),
        };
        assert_eq!(RENDERED_WITH_PATH, ps.render(&Config::default()));
    }

    #[test]
    fn without_path() {
        let ps = ComponentSection {
            id: "some-project".to_owned(),
            name: "some-project".to_owned(),
            maybe_path: None,
            entries: test_entries(),
        };
        assert_eq!(RENDERED_WITHOUT_PATH, ps.render(&Config::default()));
    }

    fn test_entries() -> Vec<Entry> {
        vec![
            Entry {
                filename: "1-issue.md".to_string(),
                id: 1,
                details: "- Issue 1".to_string(),
            },
            Entry {
                filename: "2-issue.md".to_string(),
                id: 2,
                details: "- Issue 2".to_string(),
            },
            Entry {
                filename: "3-issue.md".to_string(),
                id: 3,
                details: "- Issue 3".to_string(),
            },
        ]
    }
}
