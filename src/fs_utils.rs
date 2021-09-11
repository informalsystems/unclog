//! File system-related utilities to help with manipulating changelogs.

use crate::{Config, Error, Result};
use log::{debug, info};
use std::fs;
use std::path::{Path, PathBuf};

pub fn path_to_str<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().to_string()
}

pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

pub fn read_to_string_opt<P: AsRef<Path>>(path: P) -> Result<Option<String>> {
    let path = path.as_ref();
    if fs::metadata(path).is_err() {
        return Ok(None);
    }
    read_to_string(path).map(Some)
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    if fs::metadata(path).is_err() {
        fs::create_dir(path)?;
        info!("Created directory: {}", path_to_str(path));
    }
    if !fs::metadata(path)?.is_dir() {
        return Err(Error::ExpectedDir(path_to_str(path)));
    }
    Ok(())
}

pub fn rm_gitkeep(path: &Path) -> Result<()> {
    let path = path.join(".gitkeep");
    if fs::metadata(&path).is_ok() {
        fs::remove_file(&path)?;
        debug!("Removed .gitkeep file from: {}", path_to_str(&path));
    }
    Ok(())
}

pub fn read_and_filter_dir<F>(path: &Path, filter: F) -> Result<Vec<PathBuf>>
where
    F: Fn(fs::DirEntry) -> Option<Result<PathBuf>>,
{
    fs::read_dir(path)?
        .filter_map(|r| match r {
            Ok(e) => filter(e),
            Err(e) => Some(Err(Error::Io(e))),
        })
        .collect::<Result<Vec<PathBuf>>>()
}

pub fn entry_filter(config: &Config, e: fs::DirEntry) -> Option<Result<PathBuf>> {
    let meta = match e.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(Error::Io(e))),
    };
    let path = e.path();
    let ext = path.extension()?.to_str()?;
    if meta.is_file() && ext == config.change_sets.entry_ext {
        Some(Ok(path))
    } else {
        None
    }
}

pub fn get_relative_path<P: AsRef<Path>, Q: AsRef<Path>>(path: P, prefix: Q) -> Result<PathBuf> {
    Ok(path.as_ref().strip_prefix(prefix.as_ref())?.to_path_buf())
}

#[cfg(test)]
mod test {
    use super::get_relative_path;

    #[test]
    fn relative_path_extraction() {
        assert_eq!(
            "mypackage",
            get_relative_path("/path/to/mypackage", "/path/to")
                .unwrap()
                .to_str()
                .unwrap()
        )
    }
}
