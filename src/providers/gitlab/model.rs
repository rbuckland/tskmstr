use colored::Color;
use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use std::str::FromStr;

use crate::{
    config::{Defaults, IssueTaskRepository},
    providers::common::credentials::{CredentialKeyringEntry, HasSecretToken},
};


#[serde_inline_default]
#[derive(Debug, Deserialize, Clone)]
pub struct GitLabConfig {
    pub credential: Option<CredentialKeyringEntry>,

    #[serde_inline_default("https://gitlab.com".to_string())]
    pub endpoint: String,

    /// A String ID, used for messages
    #[serde_inline_default("gitlab.com".to_string())]
    pub provider_id: String,

    pub repositories: Vec<GitLabRepository>,
}

impl HasSecretToken for GitLabConfig {
    fn task_provider_id(&self) -> String {
        self.provider_id.clone()
    }

    fn credential(&self) -> Option<CredentialKeyringEntry> {
        self.credential.clone()
    }
}


// New type for GitLab labels
#[derive(Debug, Deserialize, Clone)]
pub struct GitLabLabel(pub String);

#[derive(Debug, Deserialize, Clone)]
pub struct GitLabIssue {
    pub iid: u32,
    pub title: String,
    pub web_url: String,

    // Use the new GitLabLabel type for tags
    pub labels: Vec<GitLabLabel>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitLabRepository {
    /// a unique character across the entire repository config
    /// which will be used for display and CMD line choices
    /// If an ID is not set, an auto generated one will be created
    pub id: String,

    /// In output, Where color is appropriate, together with the ID, this will be used
    pub color: String,

    /// the gitlab project ID, this is either the "number", or
    /// the string. If the gitlab project is a subgroup, it will look like parent%2Fchild
    pub project_id: String,

    /// Defauls configuration
    pub defaults: Option<Defaults>,

    /// filter certain issues
    pub filter: Option<String>,
}

impl IssueTaskRepository for GitLabRepository {
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
