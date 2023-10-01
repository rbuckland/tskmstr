use serde::Deserialize;
use colored::Color;
use serde_inline_default::serde_inline_default;
use std::str::FromStr;

use crate::{
    config::{Defaults, ProviderIface},
    providers::common::credentials::{CredentialKeyringEntry, HasSecretToken},
};

#[derive(Debug, Deserialize, Clone)]
pub struct JiraResult {
    pub issues: Vec<JiraIssue>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraIssue {

    #[serde(rename(deserialize = "key"))]    
    pub id: String,

    #[serde(rename(deserialize = "self"))]    
    pub url: String,
    pub fields: JiraFields,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraFields {
    pub summary: String,
    pub description: Option<JiraDescription>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraDescription {
    pub content: Vec<JiraDescriptionContent>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraDescriptionContent {
    pub content: Vec<JiraDescriptionContentItem>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraDescriptionContentItem {
    pub text: String,
}


#[serde_inline_default]
#[derive(Debug, Deserialize, Clone)]
pub struct JiraConfig {
    pub credential: Option<CredentialKeyringEntry>,

    // Required Value with your Jira endpoint URL
    pub endpoint: String,

    /// A String ID, used for messages
    #[serde_inline_default("jira".to_string())]
    pub provider_id: String,

    pub projects: Vec<JiraProject>,
}

impl HasSecretToken for JiraConfig {
    fn task_provider_id(&self) -> String {
        self.provider_id.clone()
    }

    fn credential(&self) -> Option<CredentialKeyringEntry> {
        self.credential.clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraProject {
    /// a unique character across the entire repository config
    /// which will be used for display and CMD line choices
    /// If an ID is not set, an auto generated one will be created
    pub id: String,

    /// In output, Where color is appropriate, together with the ID, this will be used
    pub color: String,

    /// the Jira project key, e.g., "PROJ123"
    pub project_key: String,

    /// Defauls configuration
    pub defaults: Option<Defaults>,
}

impl ProviderIface for JiraProject {
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
