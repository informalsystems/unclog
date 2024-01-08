use crate::changelog::fs_utils::{read_and_filter_dir, read_to_string_opt};
use crate::changelog::parsing_utils::trim_newlines;
use crate::{ChangeSetSection, Config, EntryChangeSetPath, Error, Result};
use log::debug;
use std::fs;
use std::path::{Path, PathBuf};

use super::change_set_section::ChangeSetSectionIter;

/// A set of changes, either associated with a release or not.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChangeSet {
    /// An optional high-level summary of the set of changes.
    pub maybe_summary: Option<String>,
    /// The sections making up the change set.
    pub sections: Vec<ChangeSetSection>,
}

impl ChangeSet {
    /// Returns true if this change set has no summary and no entries
    /// associated with it.
    pub fn is_empty(&self) -> bool {
        self.maybe_summary.as_ref().map_or(true, String::is_empty) && self.are_sections_empty()
    }

    /// Returns whether or not all the sections are empty.
    pub fn are_sections_empty(&self) -> bool {
        self.sections.iter().all(ChangeSetSection::is_empty)
    }

    /// Attempt to read a single change set from the given directory.
    pub fn read_from_dir<P>(config: &Config, path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        debug!("Loading change set from {}", path.display());
        let summary = read_to_string_opt(path.join(&config.change_sets.summary_filename))?
            .map(|s| trim_newlines(&s).to_owned());
        let section_dirs = read_and_filter_dir(path, change_set_section_filter)?;
        let mut sections = section_dirs
            .into_iter()
            .map(|path| ChangeSetSection::read_from_dir(config, path))
            .collect::<Result<Vec<ChangeSetSection>>>()?;
        // Sort sections alphabetically
        sections.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(Self {
            maybe_summary: summary,
            sections,
        })
    }

    /// Attempt to read a single change set from the given directory, like
    /// [`ChangeSet::read_from_dir`], but return `Option::None` if the
    /// directory does not exist.
    pub fn read_from_dir_opt<P>(config: &Config, path: P) -> Result<Option<Self>>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        // The path doesn't exist
        if fs::metadata(path).is_err() {
            return Ok(None);
        }
        Self::read_from_dir(config, path).map(Some)
    }

    pub fn render(&self, config: &Config) -> String {
        let mut paragraphs = Vec::new();
        if let Some(summary) = self.maybe_summary.as_ref() {
            paragraphs.push(summary.clone());
        }
        self.sections
            .iter()
            .filter(|s| !s.is_empty())
            .for_each(|s| paragraphs.push(s.render(config)));
        paragraphs.join("\n\n")
    }
}

#[derive(Debug, Clone)]
pub struct ChangeSetIter<'a> {
    change_set: &'a ChangeSet,
    section_id: usize,
    section_iter: ChangeSetSectionIter<'a>,
}

impl<'a> ChangeSetIter<'a> {
    pub(crate) fn new(change_set: &'a ChangeSet) -> Option<Self> {
        let first_section = change_set.sections.first()?;
        Some(Self {
            change_set,
            section_id: 0,
            section_iter: ChangeSetSectionIter::new(first_section)?,
        })
    }
}

impl<'a> Iterator for ChangeSetIter<'a> {
    type Item = EntryChangeSetPath<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let _ = self.change_set.sections.get(self.section_id)?;
        match self.section_iter.next() {
            Some(section_path) => Some(EntryChangeSetPath {
                change_set: self.change_set,
                section_path,
            }),
            None => {
                let mut maybe_section_iter = None;
                while maybe_section_iter.is_none() {
                    self.section_id += 1;
                    let section = self.change_set.sections.get(self.section_id)?;
                    maybe_section_iter = ChangeSetSectionIter::new(section);
                }
                // Safety: the above while loop will cause the function to exit
                // if we run out of sections. The while loop will only otherwise
                // terminate and hit this line if maybe_section_iter.is_none()
                // is false.
                self.section_iter = maybe_section_iter.unwrap();
                self.next()
            }
        }
    }
}

fn change_set_section_filter(entry: fs::DirEntry) -> Option<Result<PathBuf>> {
    let meta = match entry.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(Error::Io(entry.path(), e))),
    };
    if meta.is_dir() {
        Some(Ok(entry.path()))
    } else {
        None
    }
}
