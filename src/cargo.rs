//! Integration with [`cargo`](https://doc.rust-lang.org/cargo/) to facilitate
//! metadata extraction.

use crate::{Error, Result};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    manifest_path: String,
}

/// Attempt to get the manifest path for the crate with the given name from
/// within the current working directory.
pub fn get_crate_manifest_path(name: &str) -> Result<PathBuf> {
    let output = Command::new("cargo")
        .args(vec!["metadata", "--format-version=1"])
        .output()?;

    let metadata = if output.status.success() {
        String::from_utf8(output.stdout)?
    } else {
        return Err(Error::NonZeroExitCode(
            "cargo".to_owned(),
            output.status.code().unwrap(),
        ));
    };
    let metadata: Metadata = serde_json::from_str(&metadata)?;
    metadata
        .packages
        .into_iter()
        .find(|package| package.name == name)
        .map(|package| PathBuf::from(package.manifest_path))
        .ok_or_else(|| Error::NoSuchCargoPackage(name.to_owned()))
}
