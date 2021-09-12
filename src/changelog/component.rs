//! Components/sub-modules of a project.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single component of a project.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Component {
    /// The name of the component.
    pub name: String,
    /// The path of the component relative to the project path.
    pub path: PathBuf,
}
