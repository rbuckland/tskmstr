use colored::Color;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use std::str::FromStr;

use crate::{
    config::{Defaults, IssueTaskRepository},
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
    pub fields: Option<JiraFields>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraFields {
    pub summary: String,
    #[allow(dead_code)]
    pub description: Option<JiraDescription>,
    pub labels: Option<Vec<String>>,
    #[allow(dead_code)]
    pub issuetype: Option<JiraIssueType>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JiraIssueType {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraDescription {
    #[allow(dead_code)]
    pub content: Vec<JiraDescriptionContent>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraDescriptionContent {
    #[allow(dead_code)]
    pub content: Vec<JiraDescriptionContentItem>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraDescriptionContentItem {
    #[allow(dead_code)]
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

#[serde_inline_default]
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

    /// The default issue to create
    #[serde_inline_default("Task".to_string())]
    pub default_issue_type: String,

    #[serde_inline_default("1".to_string())]
    pub close_transition_id: String,

    /// Defauls configuration
    pub defaults: Option<Defaults>,

    /// filter certain issues
    pub filter: Option<String>,
}

impl IssueTaskRepository for JiraProject {
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
pub struct JiraUser {
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraStatus {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraComment {
    pub author: Option<JiraUser>,
    pub created: Option<String>,
    /// Plain text / wiki markup body (REST API v2)
    pub body: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraCommentPage {
    #[serde(default)]
    pub comments: Vec<JiraComment>,
}

/// `fields` of `GET /rest/api/2/issue/{key}` (v2 returns description and
/// comment bodies as plain strings, unlike the v3 document format)
#[derive(Debug, Deserialize, Clone)]
pub struct JiraDetailFields {
    pub summary: String,
    pub description: Option<String>,
    pub labels: Option<Vec<String>>,
    pub status: Option<JiraStatus>,
    pub reporter: Option<JiraUser>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub comment: Option<JiraCommentPage>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraIssueDetail {
    pub key: String,
    pub fields: JiraDetailFields,
}
