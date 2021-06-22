use crate::changelog::fs_utils::path_to_str;
use crate::changelog::parsing_utils::extract_release_version;
use crate::{ChangeSet, ComponentLoader, Error, Result, Version};
use log::debug;
use std::fmt;
use std::path::Path;

/// The changes associated with a specific release.
#[derive(Debug, Clone)]
pub struct Release {
    /// This release's ID (could be the version plus a prefix, e.g. `v0.1.0`).
    pub id: String,
    /// This release's version (using [semantic versioning](https://semver.org)).
    pub version: Version,
    /// The changes associated with this release.
    pub changes: ChangeSet,
}

impl Release {
    /// Attempt to read a single release from the given directory.
    pub fn read_from_dir<P, C>(path: P, component_loader: &C) -> Result<Self>
    where
        P: AsRef<Path>,
        C: ComponentLoader,
    {
        let path = path.as_ref().to_path_buf();
        debug!("Loading release from {}", path.display());
        let path_str = path_to_str(path.clone());
        if !path.is_dir() {
            return Err(Error::ExpectedDir(path_str));
        }
        let id = path
            .file_name()
            .ok_or_else(|| Error::CannotObtainName(path_str.clone()))?
            .to_string_lossy()
            .to_string();
        let version = Version::parse(extract_release_version(&id)?)?;
        Ok(Self {
            id,
            version,
            changes: ChangeSet::read_from_dir(path, component_loader)?,
        })
    }
}

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut paragraphs = vec![format!("## {}", self.id)];
        if !self.changes.is_empty() {
            paragraphs.push(self.changes.to_string());
        }
        write!(f, "{}", paragraphs.join("\n\n"))
    }
}
