//! `unclog` helps you build your changelog.

mod changelog;
mod error;

pub use changelog::{
    ChangeSet, Changelog, Entry, Release, CHANGE_SET_ENTRY_EXT, CHANGE_SET_SUMMARY_FILENAME,
    EMPTY_CHANGELOG_MSG, EPILOGUE_FILENAME, UNRELEASED_FOLDER,
};
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

// Re-exports
pub use semver::{self, Version};
