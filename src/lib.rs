//! `unclog` helps you build your changelog.

// Re-exports
pub use semver::{self, Version};

pub use changelog::{
    ChangeSet, ChangeSetSection, Changelog, Entry, Release, CHANGELOG_HEADING,
    CHANGE_SET_ENTRY_EXT, CHANGE_SET_SUMMARY_FILENAME, EMPTY_CHANGELOG_MSG, EPILOGUE_FILENAME,
    UNRELEASED_FOLDER, UNRELEASED_HEADING,
};
pub use error::Error;

mod changelog;
mod error;

pub type Result<T> = std::result::Result<T, Error>;
