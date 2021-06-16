//! Our model for a changelog.

mod change_set;
mod change_set_section;
mod entry;
mod fs_utils;
mod parsing_utils;
mod release;

use crate::{Error, Result};
use log::{debug, info};
use std::path::{Path, PathBuf};
use std::{fmt, fs};

pub use change_set::ChangeSet;
pub use change_set_section::ChangeSetSection;
pub use entry::Entry;
pub use release::Release;

pub const CHANGELOG_HEADING: &str = "# CHANGELOG";
pub const UNRELEASED_FOLDER: &str = "unreleased";
pub const UNRELEASED_HEADING: &str = "## Unreleased";
pub const EPILOGUE_FILENAME: &str = "epilogue.md";
pub const CHANGE_SET_SUMMARY_FILENAME: &str = "summary.md";
pub const CHANGE_SET_ENTRY_EXT: &str = "md";
pub const EMPTY_CHANGELOG_MSG: &str = "Nothing to see here! Add some entries to get started.";

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
    /// Checks whether this changelog is empty.
    pub fn is_empty(&self) -> bool {
        self.unreleased.as_ref().map_or(true, ChangeSet::is_empty)
            && self.releases.iter().all(|r| r.changes.is_empty())
            && self.epilogue.as_ref().map_or(true, String::is_empty)
    }

    /// Renders the full changelog to a string.
    pub fn render_full(&self) -> String {
        let mut paragraphs = vec![CHANGELOG_HEADING.to_owned()];
        if self.is_empty() {
            paragraphs.push(EMPTY_CHANGELOG_MSG.to_owned());
        } else {
            if let Ok(unreleased_paragraphs) = self.unreleased_paragraphs() {
                paragraphs.extend(unreleased_paragraphs);
            }
            self.releases
                .iter()
                .for_each(|r| paragraphs.push(r.to_string()));
            if let Some(epilogue) = self.epilogue.as_ref() {
                paragraphs.push(epilogue.clone());
            }
        }
        paragraphs.join("\n\n")
    }

    /// Renders just the unreleased changes to a string.
    pub fn render_unreleased(&self) -> Result<String> {
        Ok(self.unreleased_paragraphs()?.join("\n\n"))
    }

    fn unreleased_paragraphs(&self) -> Result<Vec<String>> {
        if let Some(unreleased) = self.unreleased.as_ref() {
            if !unreleased.is_empty() {
                return Ok(vec![UNRELEASED_HEADING.to_owned(), unreleased.to_string()]);
            }
        }
        Err(Error::NoUnreleasedEntries)
    }

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
        fs_utils::ensure_dir(path)?;

        // Optionally copy an epilogue into the target path.
        let epilogue_path = epilogue_path.as_ref();
        if let Some(ep) = epilogue_path {
            let new_epilogue_path = path.join(EPILOGUE_FILENAME);
            fs::copy(ep, &new_epilogue_path)?;
            info!(
                "Copied epilogue from {} to {}",
                fs_utils::path_to_str(ep),
                fs_utils::path_to_str(&new_epilogue_path),
            );
        }
        // We want an empty unreleased directory with a .gitkeep file
        Self::init_empty_unreleased_dir(path)?;

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
            return Err(Error::ExpectedDir(fs_utils::path_to_str(path)));
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
        let epilogue = fs_utils::read_to_string_opt(path.join(EPILOGUE_FILENAME))?
            .map(|e| parsing_utils::trim_newlines(&e).to_owned());
        Ok(Self {
            unreleased,
            releases,
            epilogue,
        })
    }

    /// Adds a changelog entry with the given ID to the specified section in
    /// the `unreleased` folder.
    pub fn add_unreleased_entry<P, S, I, C>(path: P, section: S, id: I, content: C) -> Result<()>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
        I: AsRef<str>,
        C: AsRef<str>,
    {
        let path = path.as_ref();
        let unreleased_path = path.join(UNRELEASED_FOLDER);
        fs_utils::ensure_dir(&unreleased_path)?;
        let section = section.as_ref();
        let section_path = unreleased_path.join(section);
        fs_utils::ensure_dir(&section_path)?;
        let entry_path = section_path.join(entry_id_to_filename(id));
        // We don't want to overwrite any existing entries
        if fs::metadata(&entry_path).is_ok() {
            return Err(Error::FileExists(fs_utils::path_to_str(&entry_path)));
        }
        fs::write(&entry_path, content.as_ref())?;
        info!("Wrote entry to: {}", fs_utils::path_to_str(&entry_path));
        Ok(())
    }

    /// Compute the file system path to the entry with the given parameters.
    pub fn get_entry_path<P, R, S, I>(path: P, release: R, section: S, id: I) -> PathBuf
    where
        P: AsRef<Path>,
        R: AsRef<str>,
        S: AsRef<str>,
        I: AsRef<str>,
    {
        path.as_ref()
            .join(release.as_ref())
            .join(section.as_ref())
            .join(entry_id_to_filename(id))
    }

    /// Moves the `unreleased` folder from our changelog to a directory whose
    /// name is the given version.
    pub fn prepare_release_dir<P: AsRef<Path>, S: AsRef<str>>(path: P, version: S) -> Result<()> {
        let path = path.as_ref();
        let version = version.as_ref();

        // Validate the version
        let _ = semver::Version::parse(&parsing_utils::extract_release_version(version)?)?;

        let version_path = path.join(version);
        // The target version path must not yet exist
        if fs::metadata(&version_path).is_ok() {
            return Err(Error::DirExists(fs_utils::path_to_str(&version_path)));
        }

        let unreleased_path = path.join(UNRELEASED_FOLDER);
        // The unreleased folder must exist
        if fs::metadata(&unreleased_path).is_err() {
            return Err(Error::ExpectedDir(fs_utils::path_to_str(&unreleased_path)));
        }

        fs::rename(&unreleased_path, &version_path)?;
        info!(
            "Moved {} to {}",
            fs_utils::path_to_str(&unreleased_path),
            fs_utils::path_to_str(&version_path)
        );
        // We no longer need a .gitkeep in the release directory, if there is one
        fs_utils::rm_gitkeep(&version_path)?;

        Self::init_empty_unreleased_dir(path)
    }

    fn init_empty_unreleased_dir(path: &Path) -> Result<()> {
        let unreleased_dir = path.join(UNRELEASED_FOLDER);
        fs_utils::ensure_dir(&unreleased_dir)?;
        let unreleased_gitkeep = unreleased_dir.join(".gitkeep");
        fs::write(&unreleased_gitkeep, "")?;
        debug!("Wrote {}", fs_utils::path_to_str(&unreleased_gitkeep));
        Ok(())
    }
}

impl fmt::Display for Changelog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.render_full())
    }
}

fn entry_id_to_filename<S: AsRef<str>>(id: S) -> String {
    format!("{}.{}", id.as_ref(), CHANGE_SET_ENTRY_EXT)
}

fn release_dir_filter(e: fs::DirEntry) -> Option<crate::Result<PathBuf>> {
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
