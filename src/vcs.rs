//! API for dealing with version control systems (Git) and VCS platforms (e.g.
//! GitHub).

use crate::Error;
use std::{convert::TryFrom, str::FromStr};
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

    fn try_from(url: &Url) -> Result<Self, Self::Error> {
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
            project: path_parts[1].to_owned(),
        })
    }
}

impl FromStr for GitHubProject {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s)?;
        Self::try_from(&url)
    }
}

impl std::fmt::Display for GitHubProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "https://github.com/{}/{}", self.owner, self.project)
    }
}

impl GitHubProject {
    pub fn change_url(&self, platform_id: PlatformId) -> crate::Result<Url> {
        Ok(Url::parse(&format!(
            "{}/{}",
            self.to_string(),
            match platform_id {
                PlatformId::Issue(no) => format!("issues/{}", no),
                PlatformId::PullRequest(no) => format!("pull/{}", no),
            }
        ))?)
    }
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
}
