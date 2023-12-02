use crate::changelog::fs_utils::{path_to_str, read_to_string};
use crate::changelog::parsing_utils::trim_newlines;
use crate::{Config, Error, Result};
use log::debug;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::config::SortEntriesBy;

/// A single entry in a set of changes.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The issue/pull request ID relating to this entry.
    pub id: u64,
    /// The content of the entry.
    pub details: String,
}

impl Entry {
    /// Attempt to read a single entry for a change set section from the given
    /// file.
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        debug!("Loading entry from {}", path.display());
        Ok(Self {
            id: extract_entry_id(
                path.file_name()
                    .and_then(OsStr::to_str)
                    .ok_or_else(|| Error::CannotObtainName(path_to_str(path)))?,
            )?,
            details: trim_newlines(&read_to_string(path)?).to_owned(),
        })
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.details)
    }
}

fn extract_entry_id<S: AsRef<str>>(s: S) -> Result<u64> {
    let s = s.as_ref();
    let num_digits = s
        .chars()
        .position(|c| !c.is_ascii_digit())
        .ok_or_else(|| Error::InvalidEntryId(s.to_owned()))?;
    let digits = &s[..num_digits];
    Ok(u64::from_str(digits)?)
}

pub(crate) fn read_entries_sorted(
    entry_files: Vec<PathBuf>,
    config: &Config,
) -> Result<Vec<Entry>> {
    let mut entries = entry_files
        .into_iter()
        .map(Entry::read_from_file)
        .collect::<Result<Vec<Entry>>>()?;
    // Sort entries by ID in ascending numeric order.
    entries.sort_by(|a, b| match config.change_set_sections.sort_entries_by {
        SortEntriesBy::ID => a.id.cmp(&b.id),
        SortEntriesBy::EntryText => a.details.cmp(&b.details),
    });
    Ok(entries)
}

#[cfg(test)]
mod test {
    use super::extract_entry_id;

    #[test]
    fn entry_id_extraction() {
        let cases = vec![
            ("830-something.md", 830_u64),
            ("1.md", 1_u64),
            ("0128-another-issue.md", 128_u64),
        ];

        for (s, expected) in cases {
            let actual = extract_entry_id(s).unwrap();
            assert_eq!(expected, actual);
        }

        assert!(extract_entry_id("no-number").is_err());
    }
}
