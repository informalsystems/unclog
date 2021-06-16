use crate::changelog::fs_utils::{path_to_str, read_to_string};
use crate::changelog::parsing_utils::trim_newlines;
use crate::error::Error;
use log::debug;
use std::ffi::OsStr;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

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
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let path = path.as_ref();
        debug!("Loading entry from {}", path.display());
        Ok(Self {
            id: extract_entry_id(
                path.file_name()
                    .map(OsStr::to_str)
                    .flatten()
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

fn extract_entry_id<S: AsRef<str>>(s: S) -> crate::Result<u64> {
    let s = s.as_ref();
    let num_digits = s
        .chars()
        .position(|c| !('0'..='9').contains(&c))
        .ok_or_else(|| Error::InvalidEntryId(s.to_owned()))?;
    let digits = &s[..num_digits];
    Ok(u64::from_str(digits)?)
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
