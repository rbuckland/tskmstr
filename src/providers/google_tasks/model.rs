use colored::Color;
use serde_inline_default::serde_inline_default;
use std::str::FromStr;

use serde::Deserialize;

use crate::{
    config::{Defaults, IssueTaskRepository},
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
pub struct GoogleTaskConfig {
    pub credential: Option<CredentialKeyringEntry>,

    // The google tasks API
    #[serde_inline_default("https://tasks.googleapis.com".to_string())]
    pub endpoint: String,

    /// A String ID, used for messages
    #[serde_inline_default("google_tasks".to_string())]
    pub provider_id: String,

    pub tasklists: Vec<GoogleTaskList>,
}

impl HasSecretToken for GoogleTaskConfig {

    fn task_provider_id(&self) -> String {
        self.provider_id.clone()
    }

    fn credential(&self) -> Option<CredentialKeyringEntry> {
        self.credential.clone()
    }
}

// Struct representing a task list in Google Tasks
#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleTaskList {
    pub id: String,
    pub title: String,
}

// Struct representing a task in Google Tasks
#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleTask {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub due: Option<String>,
    pub completed: Option<bool>,
}

impl IssueTaskRepository for GoogleTaskList {

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
