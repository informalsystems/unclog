//! At a high level, a changelog belongs to a project, and so we need to model
//! this accordingly.

use crate::cargo::get_crate_manifest_path;
use crate::changelog::fs_utils::get_relative_path;
use crate::{Changelog, Error, Result};
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum ProjectType {
    Rust,
}

impl ProjectType {
    /// Attempts to autodetect the type of project in the given path.
    pub fn autodetect<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if Self::is_rust_project(path)? {
            Ok(Self::Rust)
        } else {
            Err(Error::CannotAutodetectProjectType(path.to_path_buf()))
        }
    }

    fn is_rust_project(path: &Path) -> Result<bool> {
        let meta = std::fs::metadata(path.join("Cargo.toml"))?;
        if meta.is_file() {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl FromStr for ProjectType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "rust" => Ok(Self::Rust),
            _ => Err(Error::UnrecognizedProjectType(s.to_owned())),
        }
    }
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Rust => "Rust",
            }
        )
    }
}

/// A Rust project, using `cargo`.
pub type RustProject = Project<RustComponentLoader>;

/// A project, with project-specific component loader.
#[derive(Debug, Clone)]
pub struct Project<C> {
    path: PathBuf,
    component_loader: C,
}

impl<C: ComponentLoader> Project<C> {
    /// Create a project using the given path and given custom component
    /// loader.
    pub fn new_with_component_loader<P: AsRef<Path>>(path: P, component_loader: C) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            component_loader,
        }
    }

    /// Attempt to load the changelog associated with this project.
    pub fn load_changelog(&self) -> Result<Changelog> {
        Changelog::read_from_dir(&self.path, &self.component_loader)
    }
}

impl Project<RustComponentLoader> {
    /// Create a new Rust-based project.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self::new_with_component_loader(path, RustComponentLoader)
    }
}

/// A project-specific component loader.
///
/// Usually each programming language will have at least one component loader.
pub trait ComponentLoader {
    /// Attempts to load the component with the given name.
    ///
    /// If the component does not exist, this returns `Ok(None)`.
    fn get_component(&self, name: &str) -> Result<Option<Component>>;
}

/// A single component of a project.
#[derive(Debug, Clone)]
pub struct Component {
    /// The name/ID of the component.
    pub name: String,
    /// The path of the component relative to the project path.
    pub rel_path: PathBuf,
}

/// A [`ComponentLoader`] specifically for Rust-based projects.
///
/// Facilitates loading of components from the current working directory.
#[derive(Debug, Clone)]
pub struct RustComponentLoader;

impl ComponentLoader for RustComponentLoader {
    fn get_component(&self, name: &str) -> Result<Option<Component>> {
        Ok(Some(Component {
            name: name.to_owned(),
            rel_path: match get_crate_manifest_path(name) {
                Ok(abs_path) => {
                    let cwd = std::env::current_dir()?;
                    let parent_path = abs_path.parent().unwrap();
                    get_relative_path(parent_path, cwd)?
                }
                Err(Error::NoSuchCargoPackage(_)) => return Ok(None),
                Err(e) => return Err(e),
            },
        }))
    }
}
