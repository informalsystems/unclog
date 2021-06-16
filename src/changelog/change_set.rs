use crate::changelog::fs_utils::read_to_string_opt;
use crate::changelog::parsing_utils::trim_newlines;
use crate::error::Error;
use crate::{ChangeSetSection, CHANGE_SET_SUMMARY_FILENAME};
use log::debug;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

/// A set of changes, either associated with a release or not.
#[derive(Debug, Clone)]
pub struct ChangeSet {
    /// An optional high-level summary of the set of changes.
    pub summary: Option<String>,
    /// The sections making up the change set.
    pub sections: Vec<ChangeSetSection>,
}

impl ChangeSet {
    /// Returns true if this change set has no summary and no entries
    /// associated with it.
    pub fn is_empty(&self) -> bool {
        self.summary.as_ref().map_or(true, String::is_empty) && self.are_sections_empty()
    }

    /// Returns whether or not all the sections are empty.
    pub fn are_sections_empty(&self) -> bool {
        self.sections.iter().all(ChangeSetSection::is_empty)
    }

    /// Attempt to read a single change set from the given directory.
    pub fn read_from_dir<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let path = path.as_ref();
        debug!("Loading change set from {}", path.display());
        let summary = read_to_string_opt(path.join(CHANGE_SET_SUMMARY_FILENAME))?
            .map(|s| trim_newlines(&s).to_owned());
        let section_dirs = fs::read_dir(path)?
            .filter_map(|r| match r {
                Ok(e) => change_set_section_filter(e),
                Err(e) => Some(Err(Error::Io(e))),
            })
            .collect::<crate::Result<Vec<PathBuf>>>()?;
        let mut sections = section_dirs
            .into_iter()
            .map(ChangeSetSection::read_from_dir)
            .collect::<crate::Result<Vec<ChangeSetSection>>>()?;
        // Sort sections alphabetically
        sections.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(Self { summary, sections })
    }

    /// Attempt to read a single change set from the given directory, like
    /// [`ChangeSet::read_from_dir`], but return `Option::None` if the
    /// directory does not exist.
    pub fn read_from_dir_opt<P: AsRef<Path>>(path: P) -> crate::Result<Option<Self>> {
        let path = path.as_ref();
        // The path doesn't exist
        if fs::metadata(path).is_err() {
            return Ok(None);
        }
        Self::read_from_dir(path).map(Some)
    }
}

impl fmt::Display for ChangeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut paragraphs = Vec::new();
        if let Some(summary) = self.summary.as_ref() {
            paragraphs.push(summary.clone());
        }
        self.sections
            .iter()
            .filter(|s| !s.is_empty())
            .for_each(|s| paragraphs.push(s.to_string()));
        write!(f, "{}", paragraphs.join("\n\n"))
    }
}

fn change_set_section_filter(e: fs::DirEntry) -> Option<crate::Result<PathBuf>> {
    let meta = match e.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(Error::Io(e))),
    };
    if meta.is_dir() {
        Some(Ok(e.path()))
    } else {
        None
    }
}
