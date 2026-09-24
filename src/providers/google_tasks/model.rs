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
pub struct GoogleTasksOAuth2Config {
    pub client_id: String,
    pub client_secret: String,

    #[serde_inline_default("https://oauth2.googleapis.com/token".to_string())]
    pub token_endpoint: String,
}

#[serde_inline_default]
#[derive(Debug, Deserialize, Clone)]
pub struct GoogleTasksConfig {
    pub credential: Option<CredentialKeyringEntry>,

    #[serde_inline_default("https://tasks.googleapis.com".to_string())]
    pub endpoint: String,

    #[serde_inline_default("google-tasks".to_string())]
    pub provider_id: String,

    pub oauth2: GoogleTasksOAuth2Config,

    pub tasklists: Vec<GoogleTaskList>,
}

impl HasSecretToken for GoogleTasksConfig {
    fn task_provider_id(&self) -> String {
        self.provider_id.clone()
    }

    fn credential(&self) -> Option<CredentialKeyringEntry> {
        self.credential.clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct GoogleTaskList {
    pub id: String,
    pub color: String,
    pub tasklist_id: String,
    pub defaults: Option<Defaults>,
    pub filter: Option<String>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct GoogleTasksListResponse {
    #[serde(default)]
    pub items: Vec<GoogleTask>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GoogleTask {
    pub id: String,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub status: Option<String>,
    pub updated: Option<String>,
    pub due: Option<String>,
    pub completed: Option<String>,
    #[serde(rename = "selfLink")]
    pub self_link: Option<String>,
    #[serde(rename = "webViewLink")]
    pub web_view_link: Option<String>,
}
