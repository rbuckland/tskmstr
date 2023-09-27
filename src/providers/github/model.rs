use std::str::FromStr;

use serde::Deserialize;

use crate::providers::common::credentials::{HasSecretToken, CredentialKeyringEntry};

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubLabel {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubIssue {
    pub number: u32,
    pub title: String,
    pub html_url: String,

    // Use the new GitLabLabel type for tags
    pub labels: Vec<GitHubLabel>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubConfig {
    pub token: Option<String>,
    pub credential: Option<CredentialKeyringEntry>,
    pub repositories: Vec<GitHubRepository>,
}

impl HasSecretToken for GitHubConfig {
    fn task_provider_id(&self) -> String {
        "github.com".to_string()
    }

    fn token(&self) -> Option<String> {
        self.token.clone()
    }

    fn credential(&self) -> Option<CredentialKeyringEntry> {
        self.credential.clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubRepository {
    pub owner: String,
    pub repo: String,
    pub default: Option<bool>,
}
