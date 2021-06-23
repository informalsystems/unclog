//! `unclog` helps you build your changelog.

mod cargo;
mod changelog;
mod error;
mod project;

pub use changelog::{
    ChangeSet, ChangeSetSection, Changelog, ComponentSection, Entry, Release, CHANGELOG_HEADING,
    CHANGE_SET_ENTRY_EXT, CHANGE_SET_SUMMARY_FILENAME, COMPONENT_ENTRY_INDENT,
    COMPONENT_ENTRY_OVERFLOW_INDENT, COMPONENT_GENERAL_ENTRIES_TITLE, COMPONENT_NAME_PREFIX,
    EMPTY_CHANGELOG_MSG, EPILOGUE_FILENAME, UNRELEASED_FOLDER, UNRELEASED_HEADING,
};
pub use error::Error;
pub use project::{
    Component, ComponentLoader, Project, ProjectType, RustComponentLoader, RustProject,
};

/// Result type used throughout the `unclog` crate.
pub type Result<T> = std::result::Result<T, Error>;

// Re-exports
pub use semver::{self, Version};
