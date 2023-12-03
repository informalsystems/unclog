//! `unclog` helps you build your changelog.

mod changelog;
mod error;
pub mod fs_utils;
mod s11n;
mod vcs;

pub use changelog::config::{
    BulletStyle, ChangeSetsConfig, ComponentsConfig, Config, UnreleasedConfig,
};
pub use changelog::{
    ChangeSet, ChangeSetComponentPath, ChangeSetSection, ChangeSetSectionPath, Changelog,
    Component, ComponentSection, Entry, EntryChangeSetPath, EntryPath, EntryReleasePath, Release,
};
pub use error::Error;
pub use vcs::{GenericProject, PlatformId, Project};

/// Result type used throughout the `unclog` crate.
pub type Result<T> = std::result::Result<T, Error>;

// Re-exports
pub use semver::{self, Version};
