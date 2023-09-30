use std::collections::HashSet;



use colored::Color;
use serde::Deserialize;

use crate::providers::github::model::{GitHubConfig, GitHubRepository};
use crate::providers::gitlab::model::{GitLabConfig, GitLabRepository};



#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub debug: Option<bool>,

    pub colors: Colors,

    pub labels: LabelConfig,

    #[serde(rename = "github.com")]
    pub github_com: Option<GitHubConfig>,

    #[serde(rename = "gitlab.com")]
    pub gitlab_com: Option<GitLabConfig>,

}
#[derive(Debug, Deserialize, Clone)]
pub struct LabelConfig {
    pub priority_labels: HashSet<String>,
    pub priority_timeframe: Option<String>
}

/// there is only one default "place" we will create tasks into
#[derive(Debug, Deserialize, Clone)]
pub enum TaskIssueProvider {
    GitHub(GitHubRepository),
    GitLab(GitLabRepository),
}

/// Configure a task/issue source as default for some behaviour
#[derive(Debug, Deserialize, Clone)]
pub struct Defaults {
    /// One and only one Source can be set as the default for creating new tasks
    pub for_new_tasks: Option<bool>,
    /// Set this repository to show in the quick list.
    /// If this field is NOT set on any provider, then all will be displayed
    pub for_display: Option<bool>,
}

pub trait ProviderIface {

    fn defaults(&self) -> Option<Defaults>;

    fn id(&self) -> String;

    fn color(&self) -> Color;

    fn is_default(&self) -> bool {
        match self.defaults() { 
            None => false,
            Some(d) => d.for_new_tasks.unwrap_or(false)
        }
    }
}

impl AppConfig {

    pub fn find_provider_for_issue(&self, issue: &String) -> Result<Option<TaskIssueProvider>, anyhow::Error> {
        let maybe_provider: Vec<&str> = issue.split('/').collect();
        let p = String::from(*maybe_provider.first().unwrap_or_else(|
            | panic!("oops: the issue ID {} appears invalid. It was not prefixed with one of the Providers {:?}", issue, self.provider_ids())
        ));

        self.find_by(|repo: Box<&dyn ProviderIface>| repo.id() == p)
    }

    pub fn find_provider_by_id(&self, provider_id: &str) -> Result<Option<TaskIssueProvider>, anyhow::Error> {
        self.find_by(|repo: Box<&dyn ProviderIface>| repo.id() == String::from(provider_id))
    }
    
    /// Called after configuration is loaded. It determines the unique
    /// IDs for all Task/Issue providers
    // Function to get a Vec<String> of all provider IDs
    pub fn provider_ids(&self) -> Vec<String> {
        let mut provider_ids = Vec::new();

        // Check if the GitHub configuration is present
        if let Some(github_config) = &self.github_com {
            for repo in &github_config.repositories {
                provider_ids.push(repo.id.clone());
            }
        }

        // Check if the GitLab configuration is present
        if let Some(gitlab_config) = &self.gitlab_com {
            for repo in &gitlab_config.repositories {
                provider_ids.push(repo.id.clone());
            }
        }

        provider_ids
    }

    pub fn find_default_provider(&self) -> Result<Option<TaskIssueProvider>, anyhow::Error> {
        self.find_by(|repo: Box<&dyn ProviderIface>| repo.is_default())
    }

    pub fn find_by<F: Fn(Box<&dyn ProviderIface>) -> bool>(&self, f: F) -> Result<Option<TaskIssueProvider>, anyhow::Error> {
        if let Some(github_config) = &self.github_com {
            if let Some(default_repo) = github_config
                .repositories
                .iter()
                .find(|&repo| f(Box::new(repo)))
            {
                return Ok(Some(TaskIssueProvider::GitHub(default_repo.clone())));
            }
        }

        if let Some(gitlab_config) = &self.gitlab_com {
            if let Some(default_repo) = gitlab_config
                .repositories
                .iter()
                .find(|&repo| f(Box::new(repo)))
            {
                return Ok(Some(TaskIssueProvider::GitLab(default_repo.clone())));
            }
        }


        Ok(None)
    }
}

#[derive(Debug, Deserialize)]
pub struct Colors {
    pub issue_id: String,
    pub title: String,
    pub tags: String,
}

impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig {
            debug: None,
            github_com: None,
            gitlab_com: None,
            labels: LabelConfig { priority_labels: HashSet::new(), priority_timeframe: None },
            colors: Colors {
                issue_id: "magenta".to_string(),
                title: "blue".to_string(),
                tags: "green".to_string(),
            },
        }
    }
}
