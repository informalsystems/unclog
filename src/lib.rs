//! `unclog` helps you build your changelog.

mod cargo;
mod changelog;
mod error;
mod project;

pub use changelog::config::{
    BulletStyle, ChangeSetsConfig, ComponentsConfig, Config, UnreleasedConfig,
};
pub use changelog::{ChangeSet, ChangeSetSection, Changelog, ComponentSection, Entry, Release};
pub use error::Error;
pub use project::{
    Component, ComponentLoader, Project, ProjectType, RustComponentLoader, RustProject,
};

/// Result type used throughout the `unclog` crate.
pub type Result<T> = std::result::Result<T, Error>;

// Re-exports
pub use semver::{self, Version};
