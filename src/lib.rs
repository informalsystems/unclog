//! `unclog` helps you build your changelog.

mod input;
mod changelog;

pub use input::{ChangeSetInput, ChangelogInput, EntryInput, ReleaseInput};
pub use changelog::{ChangeSet, Changelog, Entry, ParseError, Release};

// Re-exports
pub use semver::{self, Version};
