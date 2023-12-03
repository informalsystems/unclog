use crate::changelog::config::SortReleasesBy;
use crate::changelog::fs_utils::path_to_str;
use crate::changelog::parsing_utils::extract_release_version;
use crate::{ChangeSet, Config, Error, Result, Version};
use chrono::NaiveDate;
use log::{debug, warn};
use std::path::Path;

/// The changes associated with a specific release.
#[derive(Debug, Clone, PartialEq)]
pub struct Release {
    /// This release's ID (could be the version plus a prefix, e.g. `v0.1.0`).
    pub id: String,
    /// This release's version (using [semantic versioning](https://semver.org)).
    pub version: Version,
    /// This possibly a release date, parsed according to the configuration file
    /// rules.
    pub maybe_date: Option<NaiveDate>,
    /// The changes associated with this release.
    pub changes: ChangeSet,
}

impl Release {
    /// Attempt to read a single release from the given directory.
    pub fn read_from_dir<P>(config: &Config, path: P) -> Result<Self>
    where
        P: AsRef<Path>,
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
        let changes = ChangeSet::read_from_dir(config, path)?;
        let maybe_date = changes.maybe_summary.as_ref().and_then(|summary| {
            let summary_first_line = match summary.split('\n').next() {
                Some(s) => s,
                None => {
                    if config.sort_releases_by.0.contains(&SortReleasesBy::Date) {
                        warn!("Unable to extract release date from {version}: unable to extract first line of summary");
                    }
                    return None
                }
            };
            for date_fmt in &config.release_date_formats.0 {
                if let Ok(date) = NaiveDate::parse_from_str(summary_first_line, date_fmt) {
                    return Some(date);
                }
            }
            if config.sort_releases_by.0.contains(&SortReleasesBy::Date) {
                warn!("Unable to parse date from first line of {version}: no formats match \"{summary_first_line}\"");
            }
            None
        });
        Ok(Self {
            id,
            version,
            maybe_date,
            changes,
        })
    }

    /// Attempt to render this release to a string using the given
    /// configuration.
    pub fn render(&self, config: &Config) -> String {
        let mut paragraphs = vec![format!("## {}", self.id)];
        if !self.changes.is_empty() {
            paragraphs.push(self.changes.render(config));
        }
        paragraphs.join("\n\n")
    }
}
