//! API for dealing with version control systems (Git) and VCS platforms (e.g.
//! GitHub).

use crate::{fs_utils::path_to_str, Error, Result};
use log::{debug, info};
use std::{convert::TryFrom, path::Path, str::FromStr};
use url::Url;

/// Provides a way of referencing a change through the VCS platform.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlatformId {
    /// The change is referenced by way of issue number.
    Issue(u32),
    /// The change is referenced by way of pull request number.
    PullRequest(u32),
}

impl PlatformId {
    /// Return the integer ID associated with this platform-specific ID.
    pub fn id(&self) -> u32 {
        match self {
            Self::Issue(issue) => *issue,
            Self::PullRequest(pull_request) => *pull_request,
        }
    }
}

/// Generic definition of an online Git project.
pub trait GenericProject {
    fn change_url(&self, platform_id: PlatformId) -> Result<Url>;
    fn url_str(&self) -> String;
    fn url(&self) -> Url;
}

impl std::fmt::Display for dyn GenericProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url_str())
    }
}

/// A project on GitHub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubProject {
    /// The organization or user associated with this project.
    pub owner: String,
    /// The ID of the project.
    pub project: String,
}

impl TryFrom<&Url> for GitHubProject {
    type Error = Error;

    fn try_from(url: &Url) -> Result<Self> {
        let host = url
            .host_str()
            .ok_or_else(|| Error::UrlMissingHost(url.to_string()))?;

        if host != "github.com" {
            return Err(Error::NotGitHubProject(url.to_string()));
        }

        let path_parts = url
            .path_segments()
            .ok_or_else(|| Error::GitHubProjectMissingPath(url.to_string()))?
            .collect::<Vec<&str>>();

        if path_parts.len() < 2 {
            return Err(Error::InvalidGitHubProjectPath(url.to_string()));
        }

        Ok(Self {
            owner: path_parts[0].to_owned(),
            project: path_parts[1].trim_end_matches(".git").to_owned(),
        })
    }
}

impl FromStr for GitHubProject {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let url = Url::parse(s)?;
        Self::try_from(&url)
    }
}

impl std::fmt::Display for GitHubProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url_str())
    }
}

impl GenericProject for GitHubProject {
    /// Construct a URL for this project based on the given platform-specific
    /// ID.
    fn change_url(&self, platform_id: PlatformId) -> Result<Url> {
        Ok(Url::parse(&format!(
            "{}/{}",
            self,
            match platform_id {
                PlatformId::Issue(no) => format!("issues/{no}"),
                PlatformId::PullRequest(no) => format!("pull/{no}"),
            }
        ))?)
    }

    fn url_str(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.project)
    }

    fn url(&self) -> Url {
        let url_str = self.url_str();
        Url::parse(&url_str).unwrap_or_else(|e| panic!("failed to parse URL \"{url_str}\": {e}"))
    }
}

/// A project on GitLab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitLabProject {
    /// The root url of the project.
    pub root_url: String,
    /// The host of the project.
    pub host: String,
    /// The ID of the project.
    pub project: String,
}

impl TryFrom<&Url> for GitLabProject {
    type Error = Error;

    fn try_from(url: &Url) -> Result<Self> {
        let host = url
            .host_str()
            .ok_or_else(|| Error::UrlMissingHost(url.to_string()))?;

        if !host.contains("gitlab") {
            return Err(Error::NotGitHubProject(url.to_string()));
        }

        let mut path_parts = url
            .path_segments()
            .ok_or_else(|| Error::GitHubProjectMissingPath(url.to_string()))?
            .collect::<Vec<&str>>();

        path_parts.retain(|&x| !x.is_empty());

        if path_parts.len() < 2 {
            return Err(Error::InvalidGitHubProjectPath(url.to_string()));
        }

        Ok(Self {
            host: host.to_owned(),
            root_url: path_parts.as_slice()[..path_parts.len() - 1]
                .to_vec()
                .join("/"),
            project: path_parts[path_parts.len() - 1]
                .trim_end_matches(".git")
                .to_owned(),
        })
    }
}

impl FromStr for GitLabProject {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let url = Url::parse(s)?;
        Self::try_from(&url)
    }
}

impl std::fmt::Display for GitLabProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url_str())
    }
}

impl GenericProject for GitLabProject {
    /// Construct a URL for this project based on the given platform-specific
    /// ID.
    fn change_url(&self, platform_id: PlatformId) -> Result<Url> {
        Ok(Url::parse(&format!(
            "{}/{}",
            self,
            match platform_id {
                PlatformId::Issue(no) => format!("-/issues/{}", no),
                PlatformId::PullRequest(no) => format!("-/merge_requests/{}", no),
            }
        ))?)
    }

    fn url_str(&self) -> String {
        format!("https://{}/{}/{}", self.host, self.root_url, self.project)
    }

    fn url(&self) -> Url {
        let url_str = self.url_str();
        Url::parse(&url_str)
            .unwrap_or_else(|e| panic!("failed to parse URL \"{}\": {}", url_str, e))
    }
}

pub enum Project {
    GitHubProject(GitHubProject),
    GitLabProject(GitLabProject),
}

impl GenericProject for Project {
    fn change_url(&self, platform_id: PlatformId) -> Result<Url> {
        match self {
            Project::GitHubProject(github) => github.change_url(platform_id),
            Project::GitLabProject(gitlab) => gitlab.change_url(platform_id),
        }
    }

    fn url_str(&self) -> String {
        match self {
            Project::GitHubProject(github) => github.url_str(),
            Project::GitLabProject(gitlab) => gitlab.url_str(),
        }
    }

    fn url(&self) -> Url {
        match self {
            Project::GitHubProject(github) => github.url(),
            Project::GitLabProject(gitlab) => gitlab.url(),
        }
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Project::GitHubProject(github) => github.fmt(f),
            Project::GitLabProject(gitlab) => gitlab.fmt(f),
        }
    }
}

pub fn from_git_repo(path: &Path, remote: &str) -> Result<Project> {
    debug!("Opening path as Git repository: {}", path_to_str(path));
    let repo = git2::Repository::open(path)?;
    let remote_url = repo
        .find_remote(remote)?
        .url()
        .map(String::from)
        .ok_or_else(|| Error::InvalidGitRemoteUrl(remote.to_owned(), path_to_str(path)))?;
    debug!("Found Git remote \"{}\" URL: {}", remote, remote_url);
    let remote_url = parse_url(&remote_url)?;
    debug!("Parsed remote URL as: {}", remote_url.to_string());

    try_from(&remote_url)
}

pub fn try_from(url: &Url) -> Result<Project> {
    if let Ok(maybe_github_project) = GitHubProject::try_from(url) {
        info!("Deduced GitHub project!");
        Ok(Project::GitHubProject(maybe_github_project))
    } else if let Ok(maybe_gitlab_project) = GitLabProject::try_from(url) {
        info!("Deduced GitLab project!");
        Ok(Project::GitLabProject(maybe_gitlab_project))
    } else {
        Err(Error::UnrecognizedProjectType(url.to_string()))
    }
}

fn parse_url(u: &str) -> Result<Url> {
    // Not an SSH URL
    if u.starts_with("http://") || u.starts_with("https://") {
        return Ok(Url::parse(u)?);
    }
    Ok(Url::parse(&format!("ssh://{}", u.replace(':', "/")))?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn github_project_url_parsing() {
        // With or without the trailing slash
        const URLS: &[&str] = &[
            "https://github.com/informalsystems/unclog",
            "https://github.com/informalsystems/unclog/",
            "https://github.com/informalsystems/unclog.git",
            "ssh://git@github.com/informalsystems/unclog.git",
        ];
        let expected = GitHubProject {
            owner: "informalsystems".to_owned(),
            project: "unclog".to_owned(),
        };
        for url in URLS {
            let actual = GitHubProject::from_str(url).unwrap();
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn github_project_url_construction() {
        let project = GitHubProject {
            owner: "informalsystems".to_owned(),
            project: "unclog".to_owned(),
        };
        assert_eq!(
            project.to_string(),
            "https://github.com/informalsystems/unclog"
        )
    }

    #[test]
    fn gitlab_project_url_parsing() {
        // With or without the trailing slash
        const URLS: &[&str] = &[
            "https://gitlab.host.com/group/project",
            "https://gitlab.host.com/group/project/",
            "https://gitlab.host.com/group/project.git",
            "ssh://git@gitlab.host.com/group/project.git",
        ];
        let expected = GitLabProject {
            root_url: "group".to_owned(),
            host: "gitlab.host.com".to_owned(),
            project: "project".to_owned(),
        };
        for url in URLS {
            let actual = GitLabProject::from_str(url).unwrap();
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn gitlab_project_url_construction() {
        let project = GitLabProject {
            root_url: "group".to_owned(),
            host: "gitlab.host.com".to_owned(),
            project: "project".to_owned(),
        };
        assert_eq!(project.to_string(), "https://gitlab.host.com/group/project")
    }
}
