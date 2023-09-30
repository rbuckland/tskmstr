use std::fmt;

use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;

use crate::config::{AppConfig, TaskIssueProvider};



use crate::providers::github::SHORT_CODE_GITHUB;
use crate::providers::gitlab::SHORT_CODE_GITLAB;

use crate::providers::o365::SHORT_CODE_O365;

#[derive(Debug, Deserialize, Clone)]
pub struct Issue {
    /// The title of the issue
    pub title: String,

    /// Originating URL
    pub html_url: String,

    /// task/ issue id referencing the foreign system
    pub id: String,

    /// List of labels, or tags
    #[serde(rename = "labels")]
    pub tags: Vec<Label>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Label {
    pub name: String,
}
