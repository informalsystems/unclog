use std::path::PathBuf;

use crate::{ChangeSet, ChangeSetSection, Changelog, ComponentSection, Config, Entry, Release};

/// Provides a precise path through a specific changelog to a specific entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryPath<'a> {
    pub changelog: &'a Changelog,
    pub release_path: EntryReleasePath<'a>,
}

impl<'a> EntryPath<'a> {
    /// Reconstructs the filesystem path, relative to the changelog folder path,
    /// for this particular entry.
    pub fn as_path(&self, config: &Config) -> PathBuf {
        self.release_path.as_path(config)
    }

    pub fn entry(&self) -> &'a Entry {
        self.release_path.entry()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryReleasePath<'a> {
    Unreleased(EntryChangeSetPath<'a>),
    Released(&'a Release, EntryChangeSetPath<'a>),
}

impl<'a> EntryReleasePath<'a> {
    pub fn as_path(&self, config: &Config) -> PathBuf {
        match self {
            Self::Unreleased(p) => PathBuf::from(&config.unreleased.folder).join(p.as_path()),
            Self::Released(r, p) => PathBuf::from(&r.id).join(p.as_path()),
        }
    }

    pub fn entry(&self) -> &'a Entry {
        match self {
            Self::Unreleased(change_set_path) => change_set_path.entry(),
            Self::Released(_, change_set_path) => change_set_path.entry(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryChangeSetPath<'a> {
    pub change_set: &'a ChangeSet,
    pub section_path: ChangeSetSectionPath<'a>,
}

impl<'a> EntryChangeSetPath<'a> {
    pub fn as_path(&self) -> PathBuf {
        self.section_path.as_path()
    }

    pub fn entry(&self) -> &'a Entry {
        self.section_path.entry()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChangeSetSectionPath<'a> {
    pub change_set_section: &'a ChangeSetSection,
    pub component_path: ChangeSetComponentPath<'a>,
}

impl<'a> ChangeSetSectionPath<'a> {
    pub fn as_path(&self) -> PathBuf {
        PathBuf::from(&self.change_set_section.id).join(self.component_path.as_path())
    }

    pub fn entry(&self) -> &'a Entry {
        self.component_path.entry()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeSetComponentPath<'a> {
    General(&'a Entry),
    Component(&'a ComponentSection, &'a Entry),
}

impl<'a> ChangeSetComponentPath<'a> {
    pub fn as_path(&self) -> PathBuf {
        match self {
            Self::General(entry) => PathBuf::from(&entry.filename),
            Self::Component(component_section, entry) => {
                PathBuf::from(&component_section.id).join(&entry.filename)
            }
        }
    }

    pub fn entry(&self) -> &'a Entry {
        match self {
            Self::General(entry) => entry,
            Self::Component(_, entry) => entry,
        }
    }
}
