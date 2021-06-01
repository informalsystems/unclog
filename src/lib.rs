//! `unclog` helps you build your changelog.

mod changelog;
mod error;

pub use changelog::{ChangeSet, Changelog, Entry, Release};
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

// Re-exports
pub use semver::{self, Version};
