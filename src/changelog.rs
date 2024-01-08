//! Our model for a changelog.

mod change_set;
mod change_set_section;
mod component;
mod component_section;
pub mod config;
mod entry;
mod entry_path;
mod parsing_utils;
mod release;

pub use change_set::ChangeSet;
pub use change_set_section::ChangeSetSection;
pub use component::Component;
pub use component_section::ComponentSection;
pub use entry::Entry;
pub use entry_path::{
    ChangeSetComponentPath, ChangeSetSectionPath, EntryChangeSetPath, EntryPath, EntryReleasePath,
};
pub use release::Release;
use serde_json::json;

use crate::changelog::config::SortReleasesBy;
use crate::changelog::parsing_utils::{extract_release_version, trim_newlines};
use crate::fs_utils::{
    self, ensure_dir, path_to_str, read_and_filter_dir, read_to_string_opt, rm_gitkeep,
};
use crate::vcs::{from_git_repo, try_from, GenericProject};
use crate::{Error, PlatformId, Result};
use config::Config;
use log::{debug, info, warn};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use self::change_set::ChangeSetIter;

const DEFAULT_CHANGE_TEMPLATE: &str =
    "{{{ bullet }}} {{{ message }}} ([\\#{{ change_id }}]({{{ change_url }}}))";

/// A log of changes for a specific project.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Changelog {
    /// Unreleased changes don't have version information associated with them.
    pub maybe_unreleased: Option<ChangeSet>,
    /// An ordered list of releases' changes.
    pub releases: Vec<Release>,
    /// Any additional content that must appear at the beginning of the
    /// changelog.
    pub prologue: Option<String>,
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
            && self.prologue.as_ref().map_or(true, String::is_empty)
            && self.epilogue.as_ref().map_or(true, String::is_empty)
    }

    /// Renders the full changelog to a string.
    pub fn render_all(&self, config: &Config) -> String {
        self.render(config, true)
    }

    /// Renders all released versions' entries, excluding unreleased ones.
    pub fn render_released(&self, config: &Config) -> String {
        self.render(config, false)
    }

    fn render(&self, config: &Config, render_unreleased: bool) -> String {
        let mut paragraphs = vec![config.heading.clone()];
        if self.is_empty() {
            paragraphs.push(config.empty_msg.clone());
        } else {
            if let Some(prologue) = self.prologue.as_ref() {
                paragraphs.push(prologue.clone());
            }
            if render_unreleased {
                if let Ok(unreleased_paragraphs) = self.unreleased_paragraphs(config) {
                    paragraphs.extend(unreleased_paragraphs);
                }
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
    pub fn init_dir<P: AsRef<Path>, R: AsRef<Path>, E: AsRef<Path>>(
        config: &Config,
        path: P,
        maybe_prologue_path: Option<R>,
        maybe_epilogue_path: Option<E>,
    ) -> Result<()> {
        let path = path.as_ref();
        // Ensure the desired path exists.
        ensure_dir(path)?;

        // Optionally copy a prologue into the target path.
        let maybe_prologue_path = maybe_prologue_path.as_ref();
        if let Some(pp) = maybe_prologue_path {
            let new_prologue_path = path.join(&config.prologue_filename);
            if !fs_utils::file_exists(&new_prologue_path) {
                fs::copy(pp, &new_prologue_path)
                    .map_err(|e| Error::Io(pp.as_ref().to_path_buf(), e))?;
                info!(
                    "Copied prologue from {} to {}",
                    path_to_str(pp),
                    path_to_str(&new_prologue_path),
                );
            } else {
                info!(
                    "Prologue file already exists, not copying: {}",
                    path_to_str(&new_prologue_path)
                );
            }
        }

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

        let maybe_git_project = if fs_utils::dir_exists(git_folder) {
            Some(from_git_repo(parent, remote.as_ref())?)
        } else {
            warn!("Parent folder of changelog directory is not a Git repository. Cannot infer whether it is a GitHub project.");
            None
        };

        let config = Config {
            maybe_project_url: maybe_git_project.map(|gp| gp.url()),
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
        releases.sort_by(|a, b| {
            for sort_by in &config.sort_releases_by.0 {
                match sort_by {
                    SortReleasesBy::Version => {
                        if a.version == b.version {
                            continue;
                        }
                        return a.version.cmp(&b.version).reverse();
                    }
                    SortReleasesBy::Date => {
                        // If either date is missing, skip to the next search
                        // criterion.
                        if a.maybe_date.is_none() || b.maybe_date.is_none() {
                            continue;
                        }
                        if a.maybe_date == b.maybe_date {
                            continue;
                        }
                        return a.maybe_date.cmp(&b.maybe_date).reverse();
                    }
                }
            }
            // Fall back to sorting by version if no sort configuration is
            // provided.
            a.version.cmp(&b.version).reverse()
        });
        let prologue = read_to_string_opt(path.join(&config.prologue_filename))?
            .map(|p| trim_newlines(&p).to_owned());
        let epilogue = read_to_string_opt(path.join(&config.epilogue_filename))?
            .map(|e| trim_newlines(&e).to_owned());
        Ok(Self {
            maybe_unreleased: unreleased,
            releases,
            prologue,
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
        Self::add_unreleased_entry(config, path, section, component, &id, rendered_change)
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
        // We only support GitHub and GitLab projects at the moment
        let git_project = try_from(project_url)?;
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
        hb.register_template_string("change", change_template)
            .map_err(|e| Error::HandlebarsTemplateLoad(e.to_string()))?;

        let (platform_id_field, platform_id_val) = match platform_id {
            PlatformId::Issue(issue) => ("issue", issue),
            PlatformId::PullRequest(pull_request) => ("pull_request", pull_request),
        };
        let template_params = json!({
            "project_url": git_project.to_string(),
            "section": section,
            "component": component,
            "id": id,
            platform_id_field: platform_id_val,
            "message": message,
            "change_url": git_project.change_url(platform_id)?.to_string(),
            "change_id": platform_id.id(),
            "bullet": config.bullet_style.to_string(),
        });
        debug!(
            "Template parameters: {}",
            serde_json::to_string_pretty(&template_params)?
        );
        let rendered_change = hb
            .render("change", &template_params)
            .map_err(|e| Error::HandlebarsTemplateRender(e.to_string()))?;
        let wrapped_rendered = textwrap::wrap(
            &rendered_change,
            textwrap::Options::new(config.wrap as usize)
                .subsequent_indent("  ")
                .break_words(false)
                .word_separator(textwrap::WordSeparator::AsciiSpace),
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

    /// Facilitates iteration through all entries in this changelog, producing
    /// [`EntryPath`] instances such that one can trace the full path to each
    /// entry. The order in which entries are produced is the order in which
    /// they will be rendered if the changelog is built.
    pub fn entries(&self) -> ChangelogEntryIter<'_> {
        if let Some(unreleased) = &self.maybe_unreleased {
            if let Some(change_set_iter) = ChangeSetIter::new(unreleased) {
                return ChangelogEntryIter {
                    changelog: self,
                    state: ChangelogEntryIterState::Unreleased(change_set_iter),
                };
            }
        }
        if let Some(releases_iter) = ReleasesIter::new(self) {
            ChangelogEntryIter {
                changelog: self,
                state: ChangelogEntryIterState::Released(releases_iter),
            }
        } else {
            ChangelogEntryIter {
                changelog: self,
                state: ChangelogEntryIterState::Empty,
            }
        }
    }

    /// Returns a list of entries that are the same across releases within this
    /// changelog. Effectively compares just the entries themselves without
    /// regard for the release, section, component, etc.
    pub fn find_duplicates_across_releases(&self) -> Vec<(EntryPath<'_>, EntryPath<'_>)> {
        let mut dups = Vec::new();
        let mut already_found = HashSet::new();

        for path_a in self.entries() {
            for path_b in self.entries() {
                if path_a == path_b {
                    continue;
                }
                if path_a.entry() == path_b.entry() && !already_found.contains(&(path_a, path_b)) {
                    dups.push((path_a, path_b));
                    already_found.insert((path_a, path_b));
                    already_found.insert((path_b, path_a));
                }
            }
        }
        dups
    }
}

#[derive(Debug, Clone)]
pub struct ChangelogEntryIter<'a> {
    changelog: &'a Changelog,
    state: ChangelogEntryIterState<'a>,
}

impl<'a> Iterator for ChangelogEntryIter<'a> {
    type Item = EntryPath<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry_release_path = self.state.next_entry_path(self.changelog)?;
        Some(EntryPath {
            changelog: self.changelog,
            release_path: entry_release_path,
        })
    }
}

#[derive(Debug, Clone)]
enum ChangelogEntryIterState<'a> {
    Empty,
    Unreleased(ChangeSetIter<'a>),
    Released(ReleasesIter<'a>),
}

impl<'a> ChangelogEntryIterState<'a> {
    fn next_entry_path(&mut self, changelog: &'a Changelog) -> Option<EntryReleasePath<'a>> {
        match self {
            Self::Empty => None,
            Self::Unreleased(change_set_iter) => match change_set_iter.next() {
                Some(entry_path) => Some(EntryReleasePath::Unreleased(entry_path)),
                None => {
                    *self = ChangelogEntryIterState::Released(ReleasesIter::new(changelog)?);
                    self.next_entry_path(changelog)
                }
            },
            Self::Released(releases_iter) => {
                let (release, entry_path) = releases_iter.next()?;
                Some(EntryReleasePath::Released(release, entry_path))
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ReleasesIter<'a> {
    releases: &'a Vec<Release>,
    id: usize,
    // Change set iterator for the current release.
    change_set_iter: ChangeSetIter<'a>,
}

impl<'a> ReleasesIter<'a> {
    fn new(changelog: &'a Changelog) -> Option<Self> {
        let releases = &changelog.releases;
        let first_release = releases.first()?;
        Some(Self {
            releases,
            id: 0,
            change_set_iter: ChangeSetIter::new(&first_release.changes)?,
        })
    }
}

impl<'a> Iterator for ReleasesIter<'a> {
    type Item = (&'a Release, EntryChangeSetPath<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut release = self.releases.get(self.id)?;
        match self.change_set_iter.next() {
            Some(entry_path) => Some((release, entry_path)),
            None => {
                let mut maybe_change_set_iter = None;
                while maybe_change_set_iter.is_none() {
                    self.id += 1;
                    release = self.releases.get(self.id)?;
                    maybe_change_set_iter = ChangeSetIter::new(&release.changes);
                }
                // Safety: the above while loop will cause the function to exit
                // if we run out of releases. The while loop will only otherwise
                // terminate and hit this line if
                // maybe_change_set_iter.is_none() is false.
                self.change_set_iter = maybe_change_set_iter.unwrap();
                self.next()
            }
        }
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
