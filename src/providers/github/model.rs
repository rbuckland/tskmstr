use colored::Color;
use serde_inline_default::serde_inline_default;
use std::str::FromStr;

use serde::Deserialize;

use crate::{
    config::{Defaults, ProviderIface},
    providers::common::credentials::{CredentialKeyringEntry, HasSecretToken},
};

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

#[serde_inline_default]
#[derive(Debug, Deserialize, Clone)]
pub struct GitHubConfig {
    pub credential: Option<CredentialKeyringEntry>,

    // The github endpoint URL
    #[serde_inline_default("https://api.github.com".to_string())]
    pub endpoint: String,

    /// A String ID, used for messages
    #[serde_inline_default("github.com".to_string())]
    pub provider_id: String,

    pub repositories: Vec<GitHubRepository>,
}

impl HasSecretToken for GitHubConfig {
    fn task_provider_id(&self) -> String {
        self.provider_id.clone()
    }

    fn credential(&self) -> Option<CredentialKeyringEntry> {
        self.credential.clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubRepository {
    /// a unique character across the entire repository config
    /// which will be used for display and CMD line choices
    /// If an ID is not set, an auto generated one will be created
    pub id: String,

    /// In output, Where color is appropriate, together with the ID, this will be used
    pub color: String,

    /// the github Owner of the repository
    pub owner: String,

    /// the github repository name
    pub repo: String,

    /// Defauls configuration
    pub defaults: Option<Defaults>,
}

impl ProviderIface for GitHubRepository {
    fn defaults(&self) -> Option<Defaults> {
        self.defaults.clone()
    }
    fn color(&self) -> Color {
        Color::from_str(&self.color).unwrap()
    }

    fn id(&self) -> String {
        self.id.clone()
    }
}
