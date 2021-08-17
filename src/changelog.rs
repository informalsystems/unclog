//! Our model for a changelog.

mod change_set;
mod change_set_section;
mod component_section;
pub mod config;
mod entry;
pub mod fs_utils;
mod parsing_utils;
mod release;

pub use change_set::ChangeSet;
pub use change_set_section::ChangeSetSection;
pub use component_section::ComponentSection;
pub use entry::Entry;
pub use release::Release;

use crate::changelog::fs_utils::{
    ensure_dir, path_to_str, read_and_filter_dir, read_to_string_opt, rm_gitkeep,
};
use crate::changelog::parsing_utils::{extract_release_version, trim_newlines};
use crate::{ComponentLoader, Error, Result};
use config::Config;
use log::{debug, info};
use std::fs;
use std::path::{Path, PathBuf};

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
        epilogue_path: Option<E>,
    ) -> Result<()> {
        let path = path.as_ref();
        // Ensure the desired path exists.
        ensure_dir(path)?;

        // Optionally copy an epilogue into the target path.
        let epilogue_path = epilogue_path.as_ref();
        if let Some(ep) = epilogue_path {
            let new_epilogue_path = path.join(&config.epilogue_filename);
            fs::copy(ep, &new_epilogue_path)?;
            info!(
                "Copied epilogue from {} to {}",
                path_to_str(ep),
                path_to_str(&new_epilogue_path),
            );
        }
        // We want an empty unreleased directory with a .gitkeep file
        Self::init_empty_unreleased_dir(config, path)?;

        info!("Success!");
        Ok(())
    }

    /// Attempt to read a full changelog from the given directory.
    pub fn read_from_dir<P, C>(config: &Config, path: P, component_loader: &mut C) -> Result<Self>
    where
        P: AsRef<Path>,
        C: ComponentLoader,
    {
        let path = path.as_ref();
        info!(
            "Attempting to load changelog from directory: {}",
            path.display()
        );
        if !fs::metadata(path)?.is_dir() {
            return Err(Error::ExpectedDir(fs_utils::path_to_str(path)));
        }
        let unreleased = ChangeSet::read_from_dir_opt(
            config,
            path.join(&config.unreleased.folder),
            component_loader,
        )?;
        debug!("Scanning for releases in {}", path.display());
        let release_dirs = read_and_filter_dir(path, |e| release_dir_filter(config, e))?;
        let mut releases = release_dirs
            .into_iter()
            .map(|path| Release::read_from_dir(config, path, component_loader))
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
        component: Option<C>,
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
        if let Some(component) = component {
            entry_dir = entry_dir.join(component.as_ref());
            ensure_dir(&entry_dir)?;
        }
        let entry_path = entry_dir.join(entry_id_to_filename(config, id));
        // We don't want to overwrite any existing entries
        if fs::metadata(&entry_path).is_ok() {
            return Err(Error::FileExists(path_to_str(&entry_path)));
        }
        fs::write(&entry_path, content.as_ref())?;
        info!("Wrote entry to: {}", path_to_str(&entry_path));
        Ok(())
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

        fs::rename(&unreleased_path, &version_path)?;
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
        fs::write(&unreleased_gitkeep, "")?;
        debug!("Wrote {}", path_to_str(&unreleased_gitkeep));
        Ok(())
    }
}

fn entry_id_to_filename<S: AsRef<str>>(config: &Config, id: S) -> String {
    format!("{}.{}", id.as_ref(), config.change_sets.entry_ext)
}

fn release_dir_filter(config: &Config, e: fs::DirEntry) -> Option<crate::Result<PathBuf>> {
    let file_name = e.file_name();
    let file_name = file_name.to_string_lossy();
    let meta = match e.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(Error::Io(e))),
    };
    if meta.is_dir() && file_name != config.unreleased.folder {
        Some(Ok(e.path()))
    } else {
        None
    }
}
