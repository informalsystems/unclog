//! Errors that can be produced by unclog.

use thiserror::Error;

/// All error variants that can be produced by unclog.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("expected path to be a directory: {0}")]
    ExpectedDir(String),
    #[error("unexpected release directory name prefix: \"{0}\"")]
    UnexpectedReleaseDirPrefix(String),
    #[error("cannot obtain (or invalid) last component of path: \"{0}\"")]
    CannotObtainName(String),
    #[error("cannot extract version")]
    CannotExtractVersion(String),
    #[error("directory already exists: {0}")]
    DirExists(String),
    #[error("file already exists: {0}")]
    FileExists(String),
    #[error("invalid semantic version")]
    InvalidSemanticVersion(#[from] semver::Error),
    #[error("expected entry ID to start with a number, but got: \"{0}\"")]
    InvalidEntryId(String),
    #[error("failed to parse entry ID as a number")]
    InvalidEntryNumber(#[from] std::num::ParseIntError),
    #[error("no unreleased entries yet")]
    NoUnreleasedEntries,
}
