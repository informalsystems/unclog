//! Configuration-related types.

use super::fs_utils::{path_to_str, read_to_string_opt};
use crate::{Error, Result};
use log::info;
use serde::{de::Error as _, Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use url::Url;

/// Configuration options relating to the generation of a changelog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The URL of the project. This helps facilitate automatic content
    /// generation when supplying an issue or PR number.
    #[serde(with = "crate::s11n::optional_from_str")]
    pub maybe_project_url: Option<Url>,
    /// The heading to use at the beginning of the changelog we generate.
    #[serde(default = "Config::default_heading")]
    pub heading: String,
    /// What style of bullet should we use when generating changelog entries?
    #[serde(with = "crate::s11n::from_str")]
    pub bullet_style: BulletStyle,
    /// The message to use when the changelog is empty.
    #[serde(default = "Config::default_empty_msg")]
    pub empty_msg: String,
    /// The filename (relative to the `.changelog` folder) of the file
    /// containing content to be appended to the end of the generated
    /// changelog.
    #[serde(default = "Config::default_epilogue_filename")]
    pub epilogue_filename: String,
    /// Configuration relating to unreleased changelog entries.
    pub unreleased: UnreleasedConfig,
    /// Configuration relating to sets of changes.
    pub change_sets: ChangeSetsConfig,
    /// Configuration relating to components/submodules.
    pub components: ComponentsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            maybe_project_url: None,
            heading: Self::default_heading(),
            bullet_style: BulletStyle::default(),
            empty_msg: Self::default_empty_msg(),
            epilogue_filename: Self::default_epilogue_filename(),
            unreleased: UnreleasedConfig::default(),
            change_sets: ChangeSetsConfig::default(),
            components: ComponentsConfig::default(),
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
            Some(content) => toml::from_str::<Self>(&content)
                .map_err(|e| Error::TomlParse(path_to_str(&path), e)),
            None => {
                info!("No changelog configuration file. Assuming defaults.");
                Ok(Self::default())
            }
        }
    }

    fn default_heading() -> String {
        "# CHANGELOG".to_owned()
    }

    fn default_empty_msg() -> String {
        "Nothing to see here! Add some entries to get started.".to_owned()
    }

    fn default_epilogue_filename() -> String {
        "epilogue.md".to_owned()
    }
}

/// The various styles of bullets available in Markdown.
#[derive(Debug, Clone, Copy)]
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
            .map_err(|e| D::Error::custom(format!("{}", e)))
    }
}

/// Configuration relating to unreleased changelog entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentsConfig {
    #[serde(default = "ComponentsConfig::default_general_entries_title")]
    pub general_entries_title: String,
    #[serde(default = "ComponentsConfig::default_entry_indent")]
    pub entry_indent: u8,
}

impl Default for ComponentsConfig {
    fn default() -> Self {
        Self {
            general_entries_title: Self::default_general_entries_title(),
            entry_indent: Self::default_entry_indent(),
        }
    }
}

impl ComponentsConfig {
    fn default_general_entries_title() -> String {
        "General".to_owned()
    }

    fn default_entry_indent() -> u8 {
        2
    }
}
