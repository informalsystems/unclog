//! Our model for a changelog.

use crate::{Error, Result};
use log::{debug, info};
use semver::Version;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fmt, fs};

const UNRELEASED_FOLDER: &str = "unreleased";
const EPILOGUE_FILENAME: &str = "epilogue.md";
const CHANGE_SET_SUMMARY_FILENAME: &str = "summary.md";
const CHANGE_SET_ENTRY_EXT: &str = "md";

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

impl Changelog {
    /// Initialize a new (empty) changelog in the given path.
    ///
    /// Creates the target folder if it doesn't exist, and optionally copies an
    /// epilogue into it.
    pub fn init_dir<P: AsRef<Path>, E: AsRef<Path>>(
        path: P,
        epilogue_path: Option<E>,
    ) -> Result<()> {
        let path = path.as_ref();
        // Ensure the desired path exists.
        if fs::metadata(path).is_err() {
            fs::create_dir(path)?;
            info!("Created directory: {}", path_to_str(path));
        }
        if !fs::metadata(path)?.is_dir() {
            return Err(Error::ExpectedDir(path_to_str(path)));
        }
        // Optionally copy an epilogue into the target path.
        let epilogue_path = epilogue_path.as_ref();
        if let Some(ep) = epilogue_path {
            let new_epilogue_path = path.join(EPILOGUE_FILENAME);
            fs::copy(ep, &new_epilogue_path)?;
            info!(
                "Copied epilogue from {} to {}",
                path_to_str(ep),
                path_to_str(&new_epilogue_path),
            );
        }
        // Ensure we at least have an unreleased directory with a .gitkeep file.
        let unreleased_dir = path.join("unreleased");
        if fs::metadata(&unreleased_dir).is_err() {
            fs::create_dir(&unreleased_dir)?;
            info!("Created directory: {}", path_to_str(&unreleased_dir));
        }
        if !fs::metadata(&unreleased_dir)?.is_dir() {
            return Err(Error::ExpectedDir(path_to_str(&unreleased_dir)));
        }
        let unreleased_gitkeep = unreleased_dir.join(".gitkeep");
        fs::write(&unreleased_gitkeep, "")?;
        debug!("Wrote {}", path_to_str(&unreleased_gitkeep));

        info!("Success!");
        Ok(())
    }

    /// Attempt to read a full changelog from the given directory.
    pub fn read_from_dir<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        info!(
            "Attempting to load changelog from directory: {}",
            path.display()
        );
        if !fs::metadata(path)?.is_dir() {
            return Err(Error::ExpectedDir(path_to_str(path)));
        }
        let unreleased = ChangeSet::read_from_dir_opt(path.join(UNRELEASED_FOLDER))?;
        debug!("Scanning for releases in {}", path.display());
        let release_dirs = fs::read_dir(path)?
            .filter_map(|r| match r {
                Ok(e) => release_dir_filter(e),
                Err(e) => Some(Err(Error::Io(e))),
            })
            .collect::<Result<Vec<PathBuf>>>()?;
        let mut releases = release_dirs
            .into_iter()
            .map(Release::read_from_dir)
            .collect::<Result<Vec<Release>>>()?;
        // Sort releases by version in descending order (newest to oldest).
        releases.sort_by(|a, b| a.version.cmp(&b.version).reverse());
        let epilogue =
            read_to_string_opt(path.join(EPILOGUE_FILENAME))?.map(|e| trim_newlines(&e).to_owned());
        Ok(Self {
            unreleased,
            releases,
            epilogue,
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

impl Release {
    /// Attempt to read a single release from the given directory.
    pub fn read_from_dir<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        debug!("Loading release from {}", path.display());
        let path_str = path_to_str(path.clone());
        if !path.is_dir() {
            return Err(Error::ExpectedDir(path_str));
        }
        let id = path
            .file_name()
            .ok_or_else(|| Error::CannotObtainName(path_str.clone()))?
            .to_string_lossy()
            .to_string();
        let version = Version::parse(extract_release_version(&id)?)?;
        Ok(Self {
            id,
            version,
            changes: ChangeSet::read_from_dir(path)?,
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

impl ChangeSet {
    /// Attempt to read a single change set from the given directory.
    pub fn read_from_dir<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        debug!("Loading change set from {}", path.display());
        let summary = read_to_string_opt(path.join(CHANGE_SET_SUMMARY_FILENAME))?
            .map(|s| trim_newlines(&s).to_owned());
        let section_dirs = fs::read_dir(path)?
            .filter_map(|r| match r {
                Ok(e) => change_set_section_filter(e),
                Err(e) => Some(Err(Error::Io(e))),
            })
            .collect::<Result<Vec<PathBuf>>>()?;
        let mut sections = section_dirs
            .into_iter()
            .map(ChangeSetSection::read_from_dir)
            .collect::<Result<Vec<ChangeSetSection>>>()?;
        // Sort sections alphabetically
        sections.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(Self { summary, sections })
    }

    /// Attempt to read a single change set from the given directory, like
    /// [`ChangeSet::read_from_dir`], but return `Option::None` if the
    /// directory does not exist.
    pub fn read_from_dir_opt<P: AsRef<Path>>(path: P) -> Result<Option<Self>> {
        let path = path.as_ref();
        // The path doesn't exist
        if fs::metadata(path).is_err() {
            return Ok(None);
        }
        Self::read_from_dir(path).map(Some)
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

impl ChangeSetSection {
    /// Attempt to read a single change set section from the given directory.
    pub fn read_from_dir<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        debug!("Loading section {}", path.display());
        let id = path
            .file_name()
            .map(OsStr::to_str)
            .flatten()
            .ok_or_else(|| Error::CannotObtainName(path_to_str(path)))?
            .to_owned();
        let title = change_set_section_title(id);
        let entry_files = fs::read_dir(path)?
            .filter_map(|r| match r {
                Ok(e) => change_set_entry_filter(e),
                Err(e) => Some(Err(Error::Io(e))),
            })
            .collect::<Result<Vec<PathBuf>>>()?;
        let mut entries = entry_files
            .into_iter()
            .map(Entry::read_from_file)
            .collect::<Result<Vec<Entry>>>()?;
        // Sort entries by ID in ascending numeric order.
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

impl Entry {
    /// Attempt to read a single entry for a change set section from the given
    /// file.
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
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

fn path_to_str<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().to_string()
}

fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

fn read_to_string_opt<P: AsRef<Path>>(path: P) -> Result<Option<String>> {
    let path = path.as_ref();
    if fs::metadata(path).is_err() {
        return Ok(None);
    }
    read_to_string(path).map(Some)
}

fn release_dir_filter(e: fs::DirEntry) -> Option<Result<PathBuf>> {
    let file_name = e.file_name();
    let file_name = file_name.to_string_lossy();
    let meta = match e.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(Error::Io(e))),
    };
    if meta.is_dir() && file_name != UNRELEASED_FOLDER {
        Some(Ok(e.path()))
    } else {
        None
    }
}

fn change_set_section_filter(e: fs::DirEntry) -> Option<Result<PathBuf>> {
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

fn change_set_entry_filter(e: fs::DirEntry) -> Option<Result<PathBuf>> {
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

fn trim_newlines(s: &str) -> &str {
    s.trim_end_matches(|c| c == '\n' || c == '\r')
}

fn change_set_section_title<S: AsRef<str>>(s: S) -> String {
    s.as_ref().to_owned().replace('-', " ").to_uppercase()
}

fn extract_entry_id<S: AsRef<str>>(s: S) -> Result<u64> {
    let s = s.as_ref();
    let num_digits = s
        .chars()
        .position(|c| !('0'..='9').contains(&c))
        .ok_or_else(|| Error::InvalidEntryId(s.to_owned()))?;
    let digits = &s[..num_digits];
    Ok(u64::from_str(digits)?)
}

fn extract_release_version(s: &str) -> Result<&str> {
    // Just find the first digit in the string
    let version_start = s
        .chars()
        .position(|c| ('0'..='9').contains(&c))
        .ok_or_else(|| Error::CannotExtractVersion(s.to_owned()))?;
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
