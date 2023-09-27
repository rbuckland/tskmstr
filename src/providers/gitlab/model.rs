use serde::Deserialize;

use crate::providers::common::credentials::{HasSecretToken, CredentialKeyringEntry};

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
pub struct GitLabConfig {
    pub token: Option<String>,
    pub credential: Option<CredentialKeyringEntry>,
    pub repositories: Vec<GitLabRepository>,
}

impl HasSecretToken for GitLabConfig {
    fn task_provider_id(&self) -> String {
       "gitlab.com".to_string()
    }

    fn token(&self) -> Option<String> {
        self.token.clone()
    }

    fn credential(&self) -> Option<CredentialKeyringEntry> {
        self.credential.clone()
    }
}


#[derive(Debug, Deserialize, Clone)]
pub struct GitLabRepository {
    pub project_id: String,
    pub default: Option<bool>,
}
