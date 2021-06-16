//! File system-related utilities to help with manipulating changelogs.

use crate::error::Error;
use log::{debug, info};
use std::fs;
use std::path::Path;

pub(crate) fn path_to_str<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().to_string()
}

pub(crate) fn read_to_string<P: AsRef<Path>>(path: P) -> crate::Result<String> {
    Ok(fs::read_to_string(path)?)
}

pub(crate) fn read_to_string_opt<P: AsRef<Path>>(path: P) -> crate::Result<Option<String>> {
    let path = path.as_ref();
    if fs::metadata(path).is_err() {
        return Ok(None);
    }
    read_to_string(path).map(Some)
}

pub(crate) fn ensure_dir(path: &Path) -> crate::Result<()> {
    if fs::metadata(path).is_err() {
        fs::create_dir(path)?;
        info!("Created directory: {}", path_to_str(path));
    }
    if !fs::metadata(path)?.is_dir() {
        return Err(Error::ExpectedDir(path_to_str(path)));
    }
    Ok(())
}

pub(crate) fn rm_gitkeep(path: &Path) -> crate::Result<()> {
    let path = path.join(".gitkeep");
    if fs::metadata(&path).is_ok() {
        fs::remove_file(&path)?;
        debug!("Removed .gitkeep file from: {}", path_to_str(&path));
    }
    Ok(())
}
