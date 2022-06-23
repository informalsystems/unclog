//! Our model for a changelog.

mod change_set;
mod change_set_section;
mod component;
mod component_section;
pub mod config;
mod entry;
mod parsing_utils;
mod release;

pub use change_set::ChangeSet;
pub use change_set_section::ChangeSetSection;
pub use component::Component;
pub use component_section::ComponentSection;
pub use entry::Entry;
pub use release::Release;
use serde_json::json;

use crate::changelog::parsing_utils::{extract_release_version, trim_newlines};
use crate::fs_utils::{
    self, ensure_dir, path_to_str, read_and_filter_dir, read_to_string_opt, rm_gitkeep,
};
use crate::{Error, GitHubProject, PlatformId, Result};
use config::Config;
use log::{debug, info, warn};
use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_CHANGE_TEMPLATE: &str =
    "{{{ bullet }}} {{{ message }}} ([#{{ change_id }}]({{{ change_url }}}))";

/// A log of changes for a specific project.
#[derive(Debug, Clone)]
pub struct Changelog {
    /// Unreleased changes don't have version information associated with them.
    pub maybe_unreleased: Option<ChangeSet>,
    /// An ordered list of releases' changes.
    pub releases: Vec<Release>,
    /// Any additional content that must appear at the end of the changelog
    /// (e.g. historical changelog content prior to switching to `unclog`).
    pub epilogue: Option<String>,
}

impl Changelog {
    /// Checks whether this changelog is empty.
    pub fn is_empty(&self) -> bool {
        self.maybe_unreleased
            .as_ref()
            .map_or(true, ChangeSet::is_empty)
            && self.releases.iter().all(|r| r.changes.is_empty())
            && self.epilogue.as_ref().map_or(true, String::is_empty)
    }

    /// Renders the full changelog to a string.
    pub fn render(&self, config: &Config) -> String {
        let mut paragraphs = vec![config.heading.clone()];
        if self.is_empty() {
            paragraphs.push(config.empty_msg.clone());
        } else {
            if let Ok(unreleased_paragraphs) = self.unreleased_paragraphs(config) {
                paragraphs.extend(unreleased_paragraphs);
            }
            self.releases
                .iter()
                .for_each(|r| paragraphs.push(r.render(config)));
            if let Some(epilogue) = self.epilogue.as_ref() {
                paragraphs.push(epilogue.clone());
            }
        }
        format!("{}\n", paragraphs.join("\n\n"))
    }

    /// Renders just the unreleased changes to a string.
    pub fn render_unreleased(&self, config: &Config) -> Result<String> {
        Ok(self.unreleased_paragraphs(config)?.join("\n\n"))
    }

    fn unreleased_paragraphs(&self, config: &Config) -> Result<Vec<String>> {
        if let Some(unreleased) = self.maybe_unreleased.as_ref() {
            if !unreleased.is_empty() {
                return Ok(vec![
                    config.unreleased.heading.clone(),
                    unreleased.render(config),
                ]);
            }
        }
        Err(Error::NoUnreleasedEntries)
    }

    /// Initialize a new (empty) changelog in the given path.
    ///
    /// Creates the target folder if it doesn't exist, and optionally copies an
    /// epilogue into it.
    pub fn init_dir<P: AsRef<Path>, E: AsRef<Path>>(
        config: &Config,
        path: P,
        maybe_epilogue_path: Option<E>,
    ) -> Result<()> {
        let path = path.as_ref();
        // Ensure the desired path exists.
        ensure_dir(path)?;

        // Optionally copy an epilogue into the target path.
        let maybe_epilogue_path = maybe_epilogue_path.as_ref();
        if let Some(ep) = maybe_epilogue_path {
            let new_epilogue_path = path.join(&config.epilogue_filename);
            if !fs_utils::file_exists(&new_epilogue_path) {
                fs::copy(ep, &new_epilogue_path)
                    .map_err(|e| Error::Io(ep.as_ref().to_path_buf(), e))?;
                info!(
                    "Copied epilogue from {} to {}",
                    path_to_str(ep),
                    path_to_str(&new_epilogue_path),
                );
            } else {
                info!(
                    "Epilogue file already exists, not copying: {}",
                    path_to_str(&new_epilogue_path)
                );
            }
        }
        // We want an empty unreleased directory with a .gitkeep file
        Self::init_empty_unreleased_dir(config, path)?;

        info!("Success!");
        Ok(())
    }

    /// Attempts to generate a configuration file for the changelog in the given
    /// path, inferring as many parameters as possible from its environment.
    pub fn generate_config<P, Q, S>(config_path: P, path: Q, remote: S, force: bool) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
        S: AsRef<str>,
    {
        let config_path = config_path.as_ref();
        if fs_utils::file_exists(config_path) {
            if !force {
                return Err(Error::ConfigurationFileAlreadyExists(path_to_str(
                    config_path,
                )));
            } else {
                warn!(
                    "Overwriting configuration file: {}",
                    path_to_str(config_path)
                );
            }
        }

        let path = fs::canonicalize(path.as_ref())
            .map_err(|e| Error::Io(path.as_ref().to_path_buf(), e))?;
        let parent = path
            .parent()
            .ok_or_else(|| Error::NoParentFolder(path_to_str(&path)))?;
        let git_folder = parent.join(".git");
        let maybe_github_project = if fs_utils::dir_exists(git_folder) {
            Some(GitHubProject::from_git_repo(parent, remote.as_ref())?)
        } else {
            warn!("Parent folder of changelog directory is not a Git repository. Cannot infer whether it is a GitHub project.");
            None
        };

        let config = Config {
            maybe_project_url: maybe_github_project.map(|gp| gp.url()),
            ..Config::default()
        };
        config.write_to_file(config_path)
    }

    /// Attempt to read a full changelog from the given directory.
    pub fn read_from_dir<P>(config: &Config, path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        info!(
            "Attempting to load changelog from directory: {}",
            path.display()
        );
        let meta = fs::metadata(path).map_err(|e| Error::Io(path.to_path_buf(), e))?;
        if !meta.is_dir() {
            return Err(Error::ExpectedDir(fs_utils::path_to_str(path)));
        }
        let unreleased =
            ChangeSet::read_from_dir_opt(config, path.join(&config.unreleased.folder))?;
        debug!("Scanning for releases in {}", path.display());
        let release_dirs = read_and_filter_dir(path, |e| release_dir_filter(config, e))?;
        let mut releases = release_dirs
            .into_iter()
            .map(|path| Release::read_from_dir(config, path))
            .collect::<Result<Vec<Release>>>()?;
        // Sort releases by version in descending order (newest to oldest).
        releases.sort_by(|a, b| a.version.cmp(&b.version).reverse());
        let epilogue = read_to_string_opt(path.join(&config.epilogue_filename))?
            .map(|e| trim_newlines(&e).to_owned());
        Ok(Self {
            maybe_unreleased: unreleased,
            releases,
            epilogue,
        })
    }

    /// Adds a changelog entry with the given ID to the specified section in
    /// the `unreleased` folder.
    pub fn add_unreleased_entry<P, S, C, I, O>(
        config: &Config,
        path: P,
        section: S,
        maybe_component: Option<C>,
        id: I,
        content: O,
    ) -> Result<()>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
        C: AsRef<str>,
        I: AsRef<str>,
        O: AsRef<str>,
    {
        let path = path.as_ref();
        let unreleased_path = path.join(&config.unreleased.folder);
        ensure_dir(&unreleased_path)?;
        let section = section.as_ref();
        let section_path = unreleased_path.join(section);
        ensure_dir(&section_path)?;
        let mut entry_dir = section_path;
        if let Some(component) = maybe_component {
            let component = component.as_ref();
            if !config.components.all.contains_key(component) {
                return Err(Error::ComponentNotDefined(component.to_string()));
            }
            entry_dir = entry_dir.join(component);
            ensure_dir(&entry_dir)?;
        }
        let entry_path = entry_dir.join(entry_id_to_filename(config, id));
        // We don't want to overwrite any existing entries
        if fs::metadata(&entry_path).is_ok() {
            return Err(Error::FileExists(path_to_str(&entry_path)));
        }
        fs::write(&entry_path, content.as_ref()).map_err(|e| Error::Io(entry_path.clone(), e))?;
        info!("Wrote entry to: {}", path_to_str(&entry_path));
        Ok(())
    }

    /// Attempts to add an unreleased changelog entry from the given parameters,
    /// rendering them through the change template specified in the
    /// configuration file.
    ///
    /// The change template is assumed to be in [Handlebars] format.
    ///
    /// [Handlebars]: https://handlebarsjs.com/
    pub fn add_unreleased_entry_from_template(
        config: &Config,
        path: &Path,
        section: &str,
        component: Option<String>,
        id: &str,
        platform_id: PlatformId,
        message: &str,
    ) -> Result<()> {
        let rendered_change = Self::render_unreleased_entry_from_template(
            config,
            path,
            section,
            component.clone(),
            id,
            platform_id,
            message,
        )?;
        let mut id = id.to_owned();
        if !id.starts_with(&format!("{}-", platform_id.id())) {
            id = format!("{}-{}", platform_id.id(), id);
            debug!("Automatically prepending platform ID to change ID: {}", id);
        }
        Self::add_unreleased_entry(config, path, section, component, &id, &rendered_change)
    }

    /// Renders an unreleased changelog entry from the given parameters to a
    /// string, making use of the change template specified in the configuration
    /// file.
    ///
    /// The change template is assumed to be in [Handlebars] format.
    ///
    /// [Handlebars]: https://handlebarsjs.com/
    pub fn render_unreleased_entry_from_template(
        config: &Config,
        path: &Path,
        section: &str,
        component: Option<String>,
        id: &str,
        platform_id: PlatformId,
        message: &str,
    ) -> Result<String> {
        let project_url = config
            .maybe_project_url
            .as_ref()
            .ok_or(Error::MissingProjectUrl)?;
        // We only support GitHub projects at the moment
        let github_project = GitHubProject::try_from(project_url)?;
        let mut change_template_file = PathBuf::from(&config.change_template);
        if change_template_file.is_relative() {
            change_template_file = path.join(change_template_file);
        }
        info!(
            "Loading change template from: {}",
            fs_utils::path_to_str(&change_template_file)
        );
        let change_template = fs_utils::read_to_string_opt(&change_template_file)?
            .unwrap_or_else(|| DEFAULT_CHANGE_TEMPLATE.to_owned());
        debug!("Loaded change template:\n{}", change_template);
        let mut hb = handlebars::Handlebars::new();
        hb.register_template_string("change", change_template)?;

        let (platform_id_field, platform_id_val) = match platform_id {
            PlatformId::Issue(issue) => ("issue", issue),
            PlatformId::PullRequest(pull_request) => ("pull_request", pull_request),
        };
        let template_params = json!({
            "project_url": github_project.to_string(),
            "section": section,
            "component": component,
            "id": id,
            platform_id_field: platform_id_val,
            "message": message,
            "change_url": github_project.change_url(platform_id)?.to_string(),
            "change_id": platform_id.id(),
            "bullet": config.bullet_style.to_string(),
        });
        debug!(
            "Template parameters: {}",
            serde_json::to_string_pretty(&template_params)?
        );
        let rendered_change = hb.render("change", &template_params)?;
        let wrapped_rendered = textwrap::wrap(
            &rendered_change,
            textwrap::Options::new(config.wrap as usize)
                .subsequent_indent("  ")
                .break_words(false)
                .word_separator(textwrap::word_separators::AsciiSpace),
        )
        .join("\n");
        debug!("Rendered wrapped change:\n{}", wrapped_rendered);
        Ok(wrapped_rendered)
    }

    /// Compute the file system path to the entry with the given parameters.
    pub fn get_entry_path<P, R, S, C, I>(
        config: &Config,
        path: P,
        release: R,
        section: S,
        component: Option<C>,
        id: I,
    ) -> PathBuf
    where
        P: AsRef<Path>,
        R: AsRef<str>,
        S: AsRef<str>,
        C: AsRef<str>,
        I: AsRef<str>,
    {
        let mut path = path.as_ref().join(release.as_ref()).join(section.as_ref());
        if let Some(component) = component {
            path = path.join(component.as_ref());
        }
        path.join(entry_id_to_filename(config, id))
    }

    /// Moves the `unreleased` folder from our changelog to a directory whose
    /// name is the given version.
    pub fn prepare_release_dir<P: AsRef<Path>, S: AsRef<str>>(
        config: &Config,
        path: P,
        version: S,
    ) -> Result<()> {
        let path = path.as_ref();
        let version = version.as_ref();

        // Validate the version
        let _ = semver::Version::parse(extract_release_version(version)?)?;

        let version_path = path.join(version);
        // The target version path must not yet exist
        if fs::metadata(&version_path).is_ok() {
            return Err(Error::DirExists(path_to_str(&version_path)));
        }

        let unreleased_path = path.join(&config.unreleased.folder);
        // The unreleased folder must exist
        if fs::metadata(&unreleased_path).is_err() {
            return Err(Error::ExpectedDir(path_to_str(&unreleased_path)));
        }

        fs::rename(&unreleased_path, &version_path)
            .map_err(|e| Error::Io(unreleased_path.clone(), e))?;
        info!(
            "Moved {} to {}",
            path_to_str(&unreleased_path),
            path_to_str(&version_path)
        );
        // We no longer need a .gitkeep in the release directory, if there is one
        rm_gitkeep(&version_path)?;

        Self::init_empty_unreleased_dir(config, path)
    }

    fn init_empty_unreleased_dir(config: &Config, path: &Path) -> Result<()> {
        let unreleased_dir = path.join(&config.unreleased.folder);
        ensure_dir(&unreleased_dir)?;
        let unreleased_gitkeep = unreleased_dir.join(".gitkeep");
        fs::write(&unreleased_gitkeep, "").map_err(|e| Error::Io(unreleased_gitkeep.clone(), e))?;
        debug!("Wrote {}", path_to_str(&unreleased_gitkeep));
        Ok(())
    }
}

fn entry_id_to_filename<S: AsRef<str>>(config: &Config, id: S) -> String {
    format!("{}.{}", id.as_ref(), config.change_sets.entry_ext)
}

fn release_dir_filter(config: &Config, entry: fs::DirEntry) -> Option<crate::Result<PathBuf>> {
    let file_name = entry.file_name();
    let file_name = file_name.to_string_lossy();
    let meta = match entry.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(Error::Io(entry.path(), e))),
    };
    if meta.is_dir() && file_name != config.unreleased.folder {
        Some(Ok(entry.path()))
    } else {
        None
    }
}
