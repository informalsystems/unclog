use crate::changelog::fs_utils;
use crate::error::Error;
use crate::Entry;
use crate::CHANGE_SET_ENTRY_EXT;
use log::debug;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

/// A single section in a set of changes.
///
/// For example, the "FEATURES" or "BREAKING CHANGES" section.
#[derive(Debug, Clone)]
pub struct ChangeSetSection {
    /// A short, descriptive title for this section (e.g. "BREAKING CHANGES").
    pub title: String,
    /// The entries in this specific set of changes.
    pub entries: Vec<Entry>,
}

impl ChangeSetSection {
    /// Returns whether or not this section is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Attempt to read a single change set section from the given directory.
    pub fn read_from_dir<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let path = path.as_ref();
        debug!("Loading section {}", path.display());
        let id = path
            .file_name()
            .map(OsStr::to_str)
            .flatten()
            .ok_or_else(|| Error::CannotObtainName(fs_utils::path_to_str(path)))?
            .to_owned();
        let title = change_set_section_title(id);
        let entry_files = fs::read_dir(path)?
            .filter_map(|r| match r {
                Ok(e) => change_set_entry_filter(e),
                Err(e) => Some(Err(Error::Io(e))),
            })
            .collect::<crate::Result<Vec<PathBuf>>>()?;
        let mut entries = entry_files
            .into_iter()
            .map(Entry::read_from_file)
            .collect::<crate::Result<Vec<Entry>>>()?;
        // Sort entries by ID in ascending numeric order.
        entries.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(Self { title, entries })
    }
}

impl fmt::Display for ChangeSetSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut entries = Vec::new();
        self.entries
            .iter()
            .for_each(|e| entries.push(e.to_string()));
        write!(f, "### {}\n\n{}", self.title, entries.join("\n"))
    }
}

fn change_set_entry_filter(e: fs::DirEntry) -> Option<crate::Result<PathBuf>> {
    let meta = match e.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(Error::Io(e))),
    };
    let path = e.path();
    let ext = path.extension()?.to_str()?;
    if meta.is_file() && ext == CHANGE_SET_ENTRY_EXT {
        Some(Ok(path))
    } else {
        None
    }
}

fn change_set_section_title<S: AsRef<str>>(s: S) -> String {
    s.as_ref().to_owned().replace('-', " ").to_uppercase()
}

#[cfg(test)]
mod test {
    use super::change_set_section_title;

    #[test]
    fn change_set_section_title_generation() {
        let cases = vec![
            ("breaking-changes", "BREAKING CHANGES"),
            ("features", "FEATURES"),
            ("improvements", "IMPROVEMENTS"),
            ("removed", "REMOVED"),
        ];

        for (s, expected) in cases {
            let actual = change_set_section_title(s);
            assert_eq!(expected, actual);
        }
    }
}
