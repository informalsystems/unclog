//! Configuration-related types.

use super::fs_utils::{path_to_str, read_to_string_opt};
use crate::{Component, Error, Result};
use log::{debug, info};
use serde::{de::Error as _, Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use url::Url;

/// Configuration options relating to the generation of a changelog.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// The URL of the project. This helps facilitate automatic content
    /// generation when supplying an issue or PR number.
    #[serde(
        default,
        rename = "project_url",
        with = "crate::s11n::optional_from_str",
        skip_serializing_if = "is_default"
    )]
    pub maybe_project_url: Option<Url>,
    /// The path to a file containing the change template to use when
    /// automatically adding new changelog entries. Relative to the `.changelog`
    /// folder.
    #[serde(
        default = "Config::default_change_template",
        skip_serializing_if = "Config::is_default_change_template"
    )]
    pub change_template: String,
    /// Wrap entries automatically to a specific number of characters per line.
    #[serde(
        default = "Config::default_wrap",
        skip_serializing_if = "Config::is_default_wrap"
    )]
    pub wrap: u16,
    /// The heading to use at the beginning of the changelog we generate.
    #[serde(
        default = "Config::default_heading",
        skip_serializing_if = "Config::is_default_heading"
    )]
    pub heading: String,
    /// What style of bullet should we use when generating changelog entries?
    #[serde(
        default,
        with = "crate::s11n::from_str",
        skip_serializing_if = "is_default"
    )]
    pub bullet_style: BulletStyle,
    /// The message to use when the changelog is empty.
    #[serde(
        default = "Config::default_empty_msg",
        skip_serializing_if = "Config::is_default_empty_msg"
    )]
    pub empty_msg: String,
    /// The filename (relative to the `.changelog` folder) of the file
    /// containing content to be inserted at the beginning of the generated
    /// changelog.
    #[serde(
        default = "Config::default_prologue_filename",
        skip_serializing_if = "Config::is_default_prologue_filename"
    )]
    pub prologue_filename: String,
    /// The filename (relative to the `.changelog` folder) of the file
    /// containing content to be appended to the end of the generated
    /// changelog.
    #[serde(
        default = "Config::default_epilogue_filename",
        skip_serializing_if = "Config::is_default_epilogue_filename"
    )]
    pub epilogue_filename: String,

    /// Sort releases by the given properties.
    #[serde(default, skip_serializing_if = "is_default")]
    pub sort_releases_by: ReleaseSortingConfig,
    /// An ordered list of possible formats to expect when parsing release
    /// summaries to establish release dates.
    #[serde(default, skip_serializing_if = "is_default")]
    pub release_date_formats: ReleaseDateFormats,

    /// Configuration relating to unreleased changelog entries.
    #[serde(default, skip_serializing_if = "is_default")]
    pub unreleased: UnreleasedConfig,
    /// Configuration relating to sets of changes.
    #[serde(default, skip_serializing_if = "is_default")]
    pub change_sets: ChangeSetsConfig,
    /// Configuration relating to change set sections.
    #[serde(default, skip_serializing_if = "is_default")]
    pub change_set_sections: ChangeSetSectionsConfig,
    /// Configuration relating to components/submodules.
    #[serde(default, skip_serializing_if = "is_default")]
    pub components: ComponentsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            maybe_project_url: None,
            change_template: Self::default_change_template(),
            wrap: Self::default_wrap(),
            heading: Self::default_heading(),
            bullet_style: BulletStyle::default(),
            empty_msg: Self::default_empty_msg(),
            prologue_filename: Self::default_prologue_filename(),
            epilogue_filename: Self::default_epilogue_filename(),
            unreleased: Default::default(),
            sort_releases_by: Default::default(),
            release_date_formats: Default::default(),
            change_sets: Default::default(),
            change_set_sections: Default::default(),
            components: Default::default(),
        }
    }
}

impl Config {
    /// Attempt to read the configuration from the given file.
    ///
    /// If the given file does not exist, this method does not fail: it returns
    /// a [`Config`] object with all of the default values set.
    ///
    /// At present, only [TOML](https://toml.io/) format is supported.
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        info!(
            "Attempting to load configuration file from: {}",
            path.display()
        );
        let maybe_content = read_to_string_opt(path)?;
        match maybe_content {
            Some(content) => {
                toml::from_str::<Self>(&content).map_err(|e| Error::TomlParse(path_to_str(path), e))
            }
            None => {
                info!("No changelog configuration file. Assuming defaults.");
                Ok(Self::default())
            }
        }
    }

    /// Attempt to save the configuration to the given file.
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        debug!(
            "Attempting to save configuration file to: {}",
            path.display()
        );
        let content = toml::to_string_pretty(&self).map_err(Error::TomlSerialize)?;
        std::fs::write(path, content).map_err(|e| Error::Io(path.to_path_buf(), e))?;
        info!("Saved configuration to: {}", path.display());
        Ok(())
    }

    fn default_change_template() -> String {
        "change-template.md".to_owned()
    }

    fn is_default_change_template(change_template: &str) -> bool {
        change_template == Self::default_change_template()
    }

    fn default_wrap() -> u16 {
        80
    }

    fn is_default_wrap(w: &u16) -> bool {
        *w == Self::default_wrap()
    }

    fn default_heading() -> String {
        "# CHANGELOG".to_owned()
    }

    fn is_default_heading(heading: &str) -> bool {
        heading == Self::default_heading()
    }

    fn default_empty_msg() -> String {
        "Nothing to see here! Add some entries to get started.".to_owned()
    }

    fn is_default_empty_msg(empty_msg: &str) -> bool {
        empty_msg == Self::default_empty_msg()
    }

    fn default_prologue_filename() -> String {
        "prologue.md".to_owned()
    }

    fn is_default_prologue_filename(prologue_filename: &str) -> bool {
        prologue_filename == Self::default_prologue_filename()
    }

    fn default_epilogue_filename() -> String {
        "epilogue.md".to_owned()
    }

    fn is_default_epilogue_filename(epilogue_filename: &str) -> bool {
        epilogue_filename == Self::default_epilogue_filename()
    }
}

/// The various styles of bullets available in Markdown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BulletStyle {
    /// `*`
    Asterisk,
    /// `-`
    Dash,
}

impl fmt::Display for BulletStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Asterisk => write!(f, "*"),
            Self::Dash => write!(f, "-"),
        }
    }
}

impl FromStr for BulletStyle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "*" => Ok(Self::Asterisk),
            "-" => Ok(Self::Dash),
            _ => Err(Error::InvalidBulletStyle),
        }
    }
}

impl Default for BulletStyle {
    fn default() -> Self {
        Self::Dash
    }
}

impl Serialize for BulletStyle {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for BulletStyle {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse::<Self>()
            .map_err(|e| D::Error::custom(format!("{e}")))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReleaseSortingConfig(pub Vec<SortReleasesBy>);

impl Default for ReleaseSortingConfig {
    fn default() -> Self {
        Self(vec![SortReleasesBy::Version])
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum SortReleasesBy {
    /// Sort releases in descending order by semantic version, with most recent
    /// version first.
    #[serde(rename = "version")]
    #[default]
    Version,
    /// Sort releases in descending order by release date, with most recent
    /// release first.
    #[serde(rename = "date")]
    Date,
}

/// An ordered list of possible release date formats to expect when parsing
/// release summaries for release dates.
///
/// See <https://docs.rs/chrono/latest/chrono/format/strftime/index.html> for
/// possible options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReleaseDateFormats(pub Vec<String>);

impl Default for ReleaseDateFormats {
    fn default() -> Self {
        Self(vec!["%F".to_string()])
    }
}

/// Configuration relating to unreleased changelog entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnreleasedConfig {
    #[serde(default = "UnreleasedConfig::default_folder")]
    pub folder: String,
    #[serde(default = "UnreleasedConfig::default_heading")]
    pub heading: String,
}

impl Default for UnreleasedConfig {
    fn default() -> Self {
        Self {
            folder: Self::default_folder(),
            heading: Self::default_heading(),
        }
    }
}

impl UnreleasedConfig {
    fn default_folder() -> String {
        "unreleased".to_owned()
    }

    fn default_heading() -> String {
        "## Unreleased".to_owned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeSetsConfig {
    #[serde(default = "ChangeSetsConfig::default_summary_filename")]
    pub summary_filename: String,
    #[serde(default = "ChangeSetsConfig::default_entry_ext")]
    pub entry_ext: String,
}

impl Default for ChangeSetsConfig {
    fn default() -> Self {
        Self {
            summary_filename: Self::default_summary_filename(),
            entry_ext: Self::default_entry_ext(),
        }
    }
}

impl ChangeSetsConfig {
    fn default_summary_filename() -> String {
        "summary.md".to_owned()
    }

    fn default_entry_ext() -> String {
        "md".to_owned()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ChangeSetSectionsConfig {
    /// Sort entries in change set sections by a specific property.
    #[serde(default, skip_serializing_if = "is_default")]
    pub sort_entries_by: SortEntriesBy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentsConfig {
    #[serde(
        default = "ComponentsConfig::default_general_entries_title",
        skip_serializing_if = "ComponentsConfig::is_default_general_entries_title"
    )]
    pub general_entries_title: String,
    #[serde(
        default = "ComponentsConfig::default_entry_indent",
        skip_serializing_if = "ComponentsConfig::is_default_entry_indent"
    )]
    pub entry_indent: u8,
    /// All of the components themselves.
    #[serde(default, skip_serializing_if = "is_default")]
    pub all: HashMap<String, Component>,
}

impl Default for ComponentsConfig {
    fn default() -> Self {
        Self {
            general_entries_title: Self::default_general_entries_title(),
            entry_indent: Self::default_entry_indent(),
            all: HashMap::default(),
        }
    }
}

impl ComponentsConfig {
    fn default_general_entries_title() -> String {
        "General".to_owned()
    }

    fn is_default_general_entries_title(t: &str) -> bool {
        t == Self::default_general_entries_title()
    }

    fn default_entry_indent() -> u8 {
        2
    }

    fn is_default_entry_indent(i: &u8) -> bool {
        *i == Self::default_entry_indent()
    }
}

fn is_default<D>(v: &D) -> bool
where
    D: Default + PartialEq,
{
    D::default().eq(v)
}

/// Allows for configuration of how entries are to be sorted within change set
/// sections.
#[derive(Debug, Clone, Default, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SortEntriesBy {
    #[serde(rename = "id")]
    #[default]
    ID,
    #[serde(rename = "entry-text")]
    EntryText,
}
