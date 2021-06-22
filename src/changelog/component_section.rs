use crate::changelog::change_set_section::indent_entries;
use crate::changelog::entry::read_entries_sorted;
use crate::changelog::fs_utils::{entry_filter, path_to_str, read_and_filter_dir};
use crate::{
    ComponentLoader, Entry, Error, Result, COMPONENT_ENTRY_INDENT, COMPONENT_ENTRY_OVERFLOW_INDENT,
    COMPONENT_NAME_PREFIX,
};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

/// A section of entries related to a specific component/submodule/package.
#[derive(Debug, Clone)]
pub struct ComponentSection {
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
    pub fn read_from_dir<P, C>(path: P, component_loader: &C) -> Result<Self>
    where
        P: AsRef<Path>,
        C: ComponentLoader,
    {
        let path = path.as_ref();
        let name = path
            .file_name()
            .map(OsStr::to_str)
            .flatten()
            .ok_or_else(|| Error::CannotObtainName(path_to_str(path)))?
            .to_owned();
        let maybe_component = component_loader.get_component(&name)?;
        let maybe_component_path = maybe_component.map(|c| c.rel_path).map(path_to_str);
        let entry_files = read_and_filter_dir(path, entry_filter)?;
        let entries = read_entries_sorted(entry_files)?;
        Ok(Self {
            name,
            maybe_path: maybe_component_path,
            entries,
        })
    }
}

impl fmt::Display for ComponentSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries_lines = indent_entries(
            &self.entries,
            COMPONENT_ENTRY_INDENT,
            COMPONENT_ENTRY_OVERFLOW_INDENT,
        );
        let name = match &self.maybe_path {
            // Render as a Markdown hyperlink
            Some(path) => format!("[{}]({})", self.name, path),
            None => self.name.clone(),
        };
        let mut lines = vec![format!("{}{}", COMPONENT_NAME_PREFIX, name)];
        lines.extend(entries_lines);
        write!(f, "{}", lines.join("\n"))
    }
}

pub(crate) fn package_section_filter(e: fs::DirEntry) -> Option<Result<PathBuf>> {
    let meta = match e.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(Error::Io(e))),
    };
    if meta.is_dir() {
        Some(Ok(e.path()))
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::ComponentSection;
    use crate::Entry;

    const RENDERED_WITH_PATH: &str = r#"* [some-project](./some-project/)
  * Issue 1
  * Issue 2
  * Issue 3"#;

    const RENDERED_WITHOUT_PATH: &str = r#"* some-project
  * Issue 1
  * Issue 2
  * Issue 3"#;

    #[test]
    fn with_path() {
        let ps = ComponentSection {
            name: "some-project".to_owned(),
            maybe_path: Some("./some-project/".to_owned()),
            entries: test_entries(),
        };
        assert_eq!(RENDERED_WITH_PATH, ps.to_string());
    }

    #[test]
    fn without_path() {
        let ps = ComponentSection {
            name: "some-project".to_owned(),
            maybe_path: None,
            entries: test_entries(),
        };
        assert_eq!(RENDERED_WITHOUT_PATH, ps.to_string());
    }

    fn test_entries() -> Vec<Entry> {
        vec![
            Entry {
                id: 1,
                details: "* Issue 1".to_string(),
            },
            Entry {
                id: 2,
                details: "* Issue 2".to_string(),
            },
            Entry {
                id: 3,
                details: "* Issue 3".to_string(),
            },
        ]
    }
}
