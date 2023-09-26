use std::collections::HashSet;
use std::hash::Hash;

use serde::Deserialize;

use crate::providers::github::model::{GitHubConfig, GitHubRepository};
use crate::providers::gitlab::model::{GitLabConfig, GitLabRepository};
use crate::providers::o365::model::{O365Config, O365TodoList};

use log::{debug, error, info, warn};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub debug: Option<bool>,

    pub colors: Colors,

    pub labels: LabelConfig,

    #[serde(rename = "github.com")]
    pub github_com: Option<GitHubConfig>,

    #[serde(rename = "gitlab.com")]
    pub gitlab_com: Option<GitLabConfig>,

    pub o365: Option<O365Config>,
}

#[derive(Debug, Deserialize)]
pub struct LabelConfig {
    pub priority_labels: HashSet<String>,
    pub priority_timeframe: Option<String>
}

/// there is only one default "place" we will create tasks into
pub enum TodoSupplier {
    GitHub(GitHubRepository),
    GitLab(GitLabRepository),
    O365(O365TodoList),
}

impl AppConfig {
    pub fn default_todo_source(&self) -> Result<Option<TodoSupplier>, anyhow::Error> {
        debug!("looking for the default Todo Provider");

        if let Some(github_config) = &self.github_com {
            if let Some(default_repo) = github_config
                .repositories
                .iter()
                .find(|&repo| repo.default.unwrap_or(false))
            {
                return Ok(Some(TodoSupplier::GitHub(default_repo.clone())));
            }
        }

        if let Some(gitlab_config) = &self.gitlab_com {
            if let Some(default_repo) = gitlab_config
                .repositories
                .iter()
                .find(|repo| repo.default.unwrap_or(false))
            {
                return Ok(Some(TodoSupplier::GitLab(default_repo.clone())));
            }
        }

        if let Some(o365_config) = &self.o365 {
            if let Some(default_todo) = o365_config
                .todo_lists
                .iter()
                .find(|list| list.default.unwrap_or(false))
            {
                return Ok(Some(TodoSupplier::O365(default_todo.clone())));
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
            o365: None,
            colors: Colors {
                issue_id: "magenta".to_string(),
                title: "blue".to_string(),
                tags: "green".to_string(),
            },
        }
    }
}
