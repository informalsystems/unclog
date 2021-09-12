use crate::changelog::component_section::package_section_filter;
use crate::changelog::entry::read_entries_sorted;
use crate::changelog::fs_utils::{entry_filter, path_to_str, read_and_filter_dir};
use crate::{ComponentSection, Config, Entry, Error, Result};
use log::debug;
use std::ffi::OsStr;
use std::path::Path;

/// A single section in a set of changes.
///
/// For example, the "FEATURES" or "BREAKING CHANGES" section.
#[derive(Debug, Clone)]
pub struct ChangeSetSection {
    /// A short, descriptive title for this section (e.g. "BREAKING CHANGES").
    pub title: String,
    /// General entries in the change set section.
    pub entries: Vec<Entry>,
    /// Entries associated with a specific component/package/submodule.
    pub component_sections: Vec<ComponentSection>,
}

impl ChangeSetSection {
    /// Returns whether or not this section is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.component_sections.is_empty()
    }

    /// Attempt to read a single change set section from the given directory.
    pub fn read_from_dir<P>(config: &Config, path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        debug!("Loading section {}", path.display());
        let id = path
            .file_name()
            .map(OsStr::to_str)
            .flatten()
            .ok_or_else(|| Error::CannotObtainName(path_to_str(path)))?
            .to_owned();
        let title = change_set_section_title(id);
        let component_section_dirs = read_and_filter_dir(path, package_section_filter)?;
        let mut component_sections = component_section_dirs
            .into_iter()
            .map(|path| ComponentSection::read_from_dir(config, path))
            .collect::<Result<Vec<ComponentSection>>>()?;
        // Component sections must be sorted by name
        component_sections.sort_by(|a, b| a.name.cmp(&b.name));
        let entry_files = read_and_filter_dir(path, |e| entry_filter(config, e))?;
        let entries = read_entries_sorted(entry_files)?;
        Ok(Self {
            title,
            entries,
            component_sections,
        })
    }

    /// Render this change set section to a string using the given
    /// configuration.
    pub fn render(&self, config: &Config) -> String {
        let mut lines = Vec::new();
        // If we have no package sections
        if self.component_sections.is_empty() {
            // Just collect the entries as-is
            lines.extend(
                self.entries
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>(),
            );
        } else {
            // If we do have package sections, however, we need to collect the
            // general entries into their own sub-section.
            if !self.entries.is_empty() {
                // For example:
                // - General
                lines.push(format!(
                    "{} {}",
                    config.bullet_style.to_string(),
                    config.components.general_entries_title
                ));
                // Now we indent all general entries.
                lines.extend(indent_entries(
                    &self.entries,
                    config.components.entry_indent,
                    config.components.entry_indent + 2,
                ));
            }
            // Component-specific sections are already indented
            lines.extend(
                self.component_sections
                    .iter()
                    .map(|ps| ps.render(config))
                    .collect::<Vec<String>>(),
            );
        }
        format!("### {}\n\n{}", self.title, lines.join("\n"))
    }
}

fn change_set_section_title<S: AsRef<str>>(s: S) -> String {
    s.as_ref().to_owned().replace('-', " ").to_uppercase()
}

// Indents the given string according to `indent` and `overflow_indent`
// assuming that the string contains one or more bulleted entries in Markdown.
fn indent_bulleted_str(s: &str, indent: u8, overflow_indent: u8) -> Vec<String> {
    s.split('\n')
        .map(|line| {
            let line_trimmed = line.trim();
            let i = if line_trimmed.starts_with('*') || line_trimmed.starts_with('-') {
                indent
            } else {
                overflow_indent
            };
            format!(
                "{}{}",
                (0..i).map(|_| " ").collect::<Vec<&str>>().join(""),
                line_trimmed
            )
        })
        .collect::<Vec<String>>()
}

pub(crate) fn indent_entries(entries: &[Entry], indent: u8, overflow_indent: u8) -> Vec<String> {
    entries
        .iter()
        .map(|e| indent_bulleted_str(e.to_string().as_str(), indent, overflow_indent))
        .flatten()
        .collect::<Vec<String>>()
}

#[cfg(test)]
mod test {
    use super::{change_set_section_title, indent_bulleted_str};

    #[test]
    fn change_set_section_title_generation() {
        let cases = vec![
            ("breaking-changes", "BREAKING CHANGES"),
            ("features", "FEATURES"),
            ("improvements", "IMPROVEMENTS"),
            ("removed", "REMOVED"),
        ];

        for (s, expected) in cases {
            let actual = change_set_section_title(s);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn entry_indentation() {
        let cases = vec![
            (
                "- Just a single-line entry.",
                "  - Just a single-line entry.",
            ),
            (
                r#"- A multi-line entry
  which overflows onto the next line."#,
                r#"  - A multi-line entry
    which overflows onto the next line."#,
            ),
            (
                r#"- A complex multi-line entry
- Which not only has multiple bulleted items
  which could overflow
- It also has bulleted items which underflow"#,
                r#"  - A complex multi-line entry
  - Which not only has multiple bulleted items
    which could overflow
  - It also has bulleted items which underflow"#,
            ),
        ];

        for (s, expected) in cases {
            let actual = indent_bulleted_str(s, 2, 4).join("\n");
            assert_eq!(expected, actual);
        }
    }
}
