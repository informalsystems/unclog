//! Our model for a changelog.

use crate::input::ChangeSetSectionInput;
use crate::{ChangeSetInput, ChangelogInput, EntryInput, ReleaseInput};
use log::debug;
use semver::Version;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;

/// Errors relating to parsing of changelog input.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("cannot extract version")]
    CannotExtractVersion(String),
    #[error("invalid semantic version")]
    InvalidSemanticVersion(#[from] semver::Error),
    #[error("expected entry ID to start with a number, but got: \"{0}\"")]
    InvalidEntryId(String),
    #[error("failed to parse entry ID as a number")]
    InvalidEntryNumber(#[from] ParseIntError),
}

/// A log of changes for a specific project.
#[derive(Debug, Clone)]
pub struct Changelog {
    /// Unreleased changes don't have version information associated with them.
    pub unreleased: Option<ChangeSet>,
    /// An ordered list of releases' changes.
    pub releases: Vec<Release>,
    /// Any additional content that must appear at the end of the changelog
    /// (e.g. historical changelog content prior to switching to `unclog`).
    pub epilogue: Option<String>,
}

impl TryFrom<ChangelogInput> for Changelog {
    type Error = ParseError;

    fn try_from(input: ChangelogInput) -> Result<Self, Self::Error> {
        let mut releases = input
            .releases
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Release>, Self::Error>>()?;
        // Sort releases by version in descending order (newest to oldest).
        releases.sort_by(|a, b| a.version.cmp(&b.version).reverse());
        Ok(Self {
            unreleased: match input.unreleased {
                Some(csi) => Some(csi.try_into()?),
                None => None,
            },
            releases,
            epilogue: input.epilogue.map(|e| trim_newlines(&e).to_owned()),
        })
    }
}

impl fmt::Display for Changelog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "# CHANGELOG\n\n{}{}{}\n",
            self.unreleased.as_ref().map_or_else(
                || "".to_owned(),
                |unreleased| format!("## Unreleased\n\n{}\n\n", unreleased)
            ),
            self.releases
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join("\n\n"),
            self.epilogue
                .as_ref()
                .map_or_else(|| "".to_owned(), Clone::clone)
        )
    }
}

/// The changes associated with a specific release.
#[derive(Debug, Clone)]
pub struct Release {
    /// This release's ID (could be the version plus a prefix, e.g. `v0.1.0`).
    pub id: String,
    /// This release's version (using [semantic versioning](https://semver.org)).
    pub version: Version,
    /// The changes associated with this release.
    pub changes: ChangeSet,
}

impl TryFrom<ReleaseInput> for Release {
    type Error = ParseError;

    fn try_from(input: ReleaseInput) -> Result<Self, Self::Error> {
        Ok(Self {
            id: input.version.clone(),
            version: Version::parse(extract_release_version(&input.version)?)?,
            changes: input.changes.try_into()?,
        })
    }
}

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "## {}\n\n{}", self.id, self.changes)
    }
}

/// A set of changes, either associated with a release or not.
#[derive(Debug, Clone)]
pub struct ChangeSet {
    /// An optional high-level summary of the set of changes.
    pub summary: Option<String>,
    /// The sections making up the change set.
    pub sections: Vec<ChangeSetSection>,
}

impl TryFrom<ChangeSetInput> for ChangeSet {
    type Error = ParseError;

    fn try_from(input: ChangeSetInput) -> Result<Self, Self::Error> {
        let (summary, sections) = (input.summary, input.sections);
        let summary = summary.map(|s| trim_newlines(&s).to_owned());
        let mut sections = sections
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<ChangeSetSection>, Self::Error>>()?;
        // Sort sections alphabetically
        sections.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(Self { summary, sections })
    }
}

impl fmt::Display for ChangeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.summary
                .as_ref()
                .map_or_else(|| "".to_owned(), |s| format!("{}\n\n", s)),
            self.sections
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join("\n\n")
        )
    }
}

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

impl TryFrom<ChangeSetSectionInput> for ChangeSetSection {
    type Error = ParseError;

    fn try_from(input: ChangeSetSectionInput) -> Result<Self, Self::Error> {
        let (title, entries) = (input.id, input.entries);
        let title = change_set_section_title(title);
        let mut entries = entries
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Entry>, Self::Error>>()?;
        // Sort entries by ID in ascending lexicographical order.
        entries.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(Self { title, entries })
    }
}

impl fmt::Display for ChangeSetSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "### {}\n\n{}",
            self.title,
            self.entries
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

/// A single entry in a set of changes.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The issue/pull request ID relating to this entry.
    pub id: u64,
    /// The content of the entry.
    pub details: String,
}

impl TryFrom<EntryInput> for Entry {
    type Error = ParseError;

    fn try_from(input: EntryInput) -> Result<Self, Self::Error> {
        debug!("Parsing entry from {}", input.id);
        Ok(Self {
            id: extract_entry_id(input.id)?,
            details: trim_newlines(&input.details).to_owned(),
        })
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.details)
    }
}

fn trim_newlines(s: &str) -> &str {
    s.trim_end_matches(|c| c == '\n' || c == '\r')
}

fn change_set_section_title<S: AsRef<str>>(s: S) -> String {
    s.as_ref().to_owned().replace('-', " ").to_uppercase()
}

fn extract_entry_id<S: AsRef<str>>(s: S) -> Result<u64, ParseError> {
    let s = s.as_ref();
    let num_digits = s
        .chars()
        .position(|c| !('0'..='9').contains(&c))
        .ok_or_else(|| ParseError::InvalidEntryId(s.to_owned()))?;
    let digits = &s[..num_digits];
    Ok(u64::from_str(digits)?)
}

fn extract_release_version(s: &str) -> Result<&str, ParseError> {
    // Just find the first digit in the string
    let version_start = s
        .chars()
        .position(|c| ('0'..='9').contains(&c))
        .ok_or_else(|| ParseError::CannotExtractVersion(s.to_owned()))?;
    Ok(&s[version_start..])
}

#[cfg(test)]
mod test {
    use super::{change_set_section_title, extract_entry_id, extract_release_version};

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

    #[test]
    fn release_version_extraction() {
        let cases = vec![
            ("v0.1.0", "0.1.0"),
            ("0.1.0", "0.1.0"),
            ("v0.1.0-beta.1", "0.1.0-beta.1"),
        ];

        for (s, expected) in cases {
            let actual = extract_release_version(s).unwrap();
            assert_eq!(expected, actual);
        }

        assert!(extract_release_version("no-version").is_err());
    }
}
