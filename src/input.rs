//! Preprocessed input that can be parsed to produce a [`Changelog`].

use log::{debug, info};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const UNRELEASED_FOLDER: &str = "unreleased";
const EPILOGUE_FILENAME: &str = "epilogue.md";
const CHANGE_SET_SUMMARY_FILENAME: &str = "summary.md";
const CHANGE_SET_ENTRY_EXT: &str = "md";

/// Errors relating to interaction with the file system.
#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("expected path to be a directory: {0}")]
    ExpectedDir(String),
    #[error("unexpected release directory name prefix: \"{0}\"")]
    UnexpectedReleaseDirPrefix(String),
    #[error("cannot obtain (or invalid) last component of path: \"{0}\"")]
    CannotObtainName(String),
}

/// Partially processed input that somewhat resembles a [`Changelog`], but has
/// not yet been fully parsed and sorted.
#[derive(Debug, Clone)]
pub struct ChangelogInput {
    pub unreleased: Option<ChangeSetInput>,
    pub releases: Vec<ReleaseInput>,
    pub epilogue: Option<String>,
}

impl ChangelogInput {
    /// Attempt to load a partially processed changelog from the given directory.
    pub fn load_from_dir<P: AsRef<Path>>(path: P) -> Result<Self, FileSystemError> {
        let path = path.as_ref();
        info!(
            "Attempting to load changelog from directory: {}",
            path.display()
        );
        if !fs::metadata(path)?.is_dir() {
            return Err(FileSystemError::ExpectedDir(path_to_str(path)));
        }
        let unreleased = ChangeSetInput::load_from_dir_opt(path.join(UNRELEASED_FOLDER))?;
        debug!("Scanning for releases in {}", path.display());
        let release_dirs = fs::read_dir(path)?
            .filter_map(|r| match r {
                Ok(e) => release_dir_filter(e),
                Err(e) => Some(Err(FileSystemError::Io(e))),
            })
            .collect::<Result<Vec<PathBuf>, FileSystemError>>()?;
        // We don't sort at this point, because that would require parsing.
        let releases = release_dirs
            .into_iter()
            .map(ReleaseInput::load_from_dir)
            .collect::<Result<Vec<ReleaseInput>, FileSystemError>>()?;
        let epilogue = read_to_string_opt(path.join(EPILOGUE_FILENAME))?;
        Ok(Self {
            unreleased,
            releases,
            epilogue,
        })
    }
}

/// Partially processed release content.
#[derive(Debug, Clone)]
pub struct ReleaseInput {
    pub version: String,
    pub changes: ChangeSetInput,
}

impl ReleaseInput {
    /// Attempt to load partially processed release information from the given directory.
    pub fn load_from_dir<P: AsRef<Path>>(path: P) -> Result<Self, FileSystemError> {
        let path = path.as_ref().to_path_buf();
        debug!("Loading release from {}", path.display());
        let path_str = path_to_str(path.clone());
        if !path.is_dir() {
            return Err(FileSystemError::ExpectedDir(path_str));
        }
        let version = path
            .file_name()
            .ok_or_else(|| FileSystemError::CannotObtainName(path_str.clone()))?
            .to_string_lossy()
            .to_string();
        Ok(Self {
            version,
            changes: ChangeSetInput::load_from_dir(path)?,
        })
    }
}

/// A partially processed change set.
#[derive(Debug, Clone)]
pub struct ChangeSetInput {
    pub summary: Option<String>,
    pub sections: Vec<ChangeSetSectionInput>,
}

impl Default for ChangeSetInput {
    fn default() -> Self {
        Self {
            summary: None,
            sections: vec![],
        }
    }
}

impl ChangeSetInput {
    /// Attempt to load a partially processed set of changes from the given directory.
    pub fn load_from_dir<P: AsRef<Path>>(path: P) -> Result<Self, FileSystemError> {
        let path = path.as_ref();
        debug!("Loading change set from {}", path.display());
        let summary = read_to_string_opt(path.join(CHANGE_SET_SUMMARY_FILENAME))?;
        let section_dirs = fs::read_dir(path)?
            .filter_map(|r| match r {
                Ok(e) => change_set_section_filter(e),
                Err(e) => Some(Err(FileSystemError::Io(e))),
            })
            .collect::<Result<Vec<PathBuf>, FileSystemError>>()?;
        let sections = section_dirs
            .into_iter()
            .map(ChangeSetSectionInput::load_from_path)
            .collect::<Result<Vec<ChangeSetSectionInput>, FileSystemError>>()?;
        Ok(Self { summary, sections })
    }

    /// Attempt to load a partially processed set of changes from the given directory,
    /// but if the directory does not exist, return `Ok(None)` instead of an error.
    pub fn load_from_dir_opt<P: AsRef<Path>>(path: P) -> Result<Option<Self>, FileSystemError> {
        let path = path.as_ref();
        // The path doesn't exist
        if fs::metadata(path).is_err() {
            return Ok(None);
        }
        Self::load_from_dir(path).map(Some)
    }
}

/// A partially processed section within a change set (e.g. the "BREAKING FEATURES" section).
#[derive(Debug, Clone)]
pub struct ChangeSetSectionInput {
    pub id: String,
    pub entries: Vec<EntryInput>,
}

impl ChangeSetSectionInput {
    /// Attempt to load a partially processed change set section from the given directory.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, FileSystemError> {
        let path = path.as_ref();
        debug!("Loading section {}", path.display());
        let id = path
            .file_name()
            .map(OsStr::to_str)
            .flatten()
            .ok_or_else(|| FileSystemError::CannotObtainName(path_to_str(path)))?
            .to_owned();
        let entry_files = fs::read_dir(path)?
            .filter_map(|r| match r {
                Ok(e) => change_set_entry_filter(e),
                Err(e) => Some(Err(FileSystemError::Io(e))),
            })
            .collect::<Result<Vec<PathBuf>, FileSystemError>>()?;
        let entries = entry_files
            .into_iter()
            .map(EntryInput::load_from_file)
            .collect::<Result<Vec<EntryInput>, FileSystemError>>()?;
        Ok(Self { id, entries })
    }
}

/// A partially processed changelog entry.
#[derive(Debug, Clone)]
pub struct EntryInput {
    pub id: String,
    pub details: String,
}

impl EntryInput {
    /// Attempt to load a partially processed changelog entry from the given file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, FileSystemError> {
        let path = path.as_ref();
        debug!("Loading entry from {}", path.display());
        Ok(Self {
            id: path
                .file_name()
                .map(OsStr::to_str)
                .flatten()
                .ok_or_else(|| FileSystemError::CannotObtainName(path_to_str(path)))?
                .to_owned(),
            details: read_to_string(path)?,
        })
    }
}

fn path_to_str<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().to_string()
}

fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, FileSystemError> {
    Ok(fs::read_to_string(path)?)
}

fn read_to_string_opt<P: AsRef<Path>>(path: P) -> Result<Option<String>, FileSystemError> {
    let path = path.as_ref();
    if fs::metadata(path).is_err() {
        return Ok(None);
    }
    read_to_string(path).map(Some)
}

fn release_dir_filter(e: fs::DirEntry) -> Option<Result<PathBuf, FileSystemError>> {
    let file_name = e.file_name();
    let file_name = file_name.to_string_lossy();
    let meta = match e.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(FileSystemError::Io(e))),
    };
    if meta.is_dir() && file_name != UNRELEASED_FOLDER {
        Some(Ok(e.path()))
    } else {
        None
    }
}

fn change_set_section_filter(e: fs::DirEntry) -> Option<Result<PathBuf, FileSystemError>> {
    let meta = match e.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(FileSystemError::Io(e))),
    };
    if meta.is_dir() {
        Some(Ok(e.path()))
    } else {
        None
    }
}

fn change_set_entry_filter(e: fs::DirEntry) -> Option<Result<PathBuf, FileSystemError>> {
    let meta = match e.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(FileSystemError::Io(e))),
    };
    let path = e.path();
    let ext = path.extension()?.to_str()?;
    if meta.is_file() && ext == CHANGE_SET_ENTRY_EXT {
        Some(Ok(path))
    } else {
        None
    }
}
