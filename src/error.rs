//! Errors that can be produced by unclog.

use std::path::PathBuf;
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
    #[error("non-UTF8 characters in string")]
    NonUtf8String(#[from] std::string::FromUtf8Error),
    #[error("non-zero process exit code when executing {0}: {1}")]
    NonZeroExitCode(String, i32),
    #[error("failed to parse JSON: {0}")]
    JsonParsingFailed(#[from] serde_json::Error),
    #[error("no such cargo package: {0}")]
    NoSuchCargoPackage(String),
    #[error("failed to get relative package path: {0}")]
    StripPrefixError(#[from] std::path::StripPrefixError),
    #[error("unrecognized project type: {0}")]
    UnrecognizedProjectType(String),
    #[error("cannot autodetect project type in path: {0}")]
    CannotAutodetectProjectType(PathBuf),
    #[error("invalid bullet style - can only be \"*\" or \"-\"")]
    InvalidBulletStyle,
    #[error("failed to parse TOML file \"{0}\": {1}")]
    TomlParse(String, toml::de::Error),
    #[error("failed to serialize TOML: {0}")]
    TomlSerialize(toml::ser::Error),
}
