use anyhow::Result;
use either::*;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;

use crate::config::AppConfig;
use crate::providers::gitlab::model::GitLabRepository;
use crate::providers::github::model::GitHubRepository;

use crate::providers::gitlab::SHORT_CODE_GITLAB;
use crate::providers::github::SHORT_CODE_GITHUB;



#[derive(Debug, Deserialize, Clone)]
pub enum TodoSource {
    GitHub(u16, String), // unique idx, issue_id
    GitLab(u16, String), // unique idx, issue_id
}

lazy_static! {
    /// Regex for tskmstr representations of a task across all task providers
    pub static ref TASK_ID_RE: Regex = Regex::new(format!(r"^(?<task_source>{}|{})(?<todosrc_idx>[0-9]+)/(?<issue_id>[A-Za-z0-9_-]+)$",SHORT_CODE_GITHUB, SHORT_CODE_GITLAB).as_str()).unwrap();
}

impl std::str::FromStr for TodoSource {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(task_id) = TASK_ID_RE.captures(s) {
            match &task_id["task_source"] {
                "gh" => {
                    let todosrc_idx = u16::from_str_radix(&task_id["todosrc_idx"], 10).unwrap();
                    Ok(TodoSource::GitHub(
                        todosrc_idx,
                        task_id["issue_id"].to_string(),
                    ))
                }
                "gl" => {
                    let todosrc_idx = u16::from_str_radix(&task_id["todosrc_idx"], 10).unwrap();
                    Ok(TodoSource::GitLab(
                        todosrc_idx,
                        task_id["issue_id"].to_string(),
                    ))
                }
                _ => Err(anyhow::anyhow!("Invalid task source")),
            }
        } else {
            Err(anyhow::anyhow!("Invalid task ID format"))
        }
    }
}

impl TodoSource {
    pub fn task_supplier<'a>(
        self,
        app_config: &'a AppConfig,
    ) -> Either<&'a GitHubRepository, &'a GitLabRepository> {
        match self {
            TodoSource::GitHub(todosrc_idx, _) => {
                let github_config = &app_config.github_com;
                let github_repo = github_config
                    .as_ref()
                    .and_then(|config| config.repositories.get(todosrc_idx as usize));
                Either::Left(github_repo.expect("Invalid GitHub index"))
            }
            TodoSource::GitLab(todosrc_idx, _) => {
                let gitlab_config = &app_config.gitlab_com;
                let gitlab_repo = gitlab_config
                    .as_ref()
                    .and_then(|config| config.repositories.get(todosrc_idx as usize));
                Either::Right(gitlab_repo.expect("Invalid GitLab index"))
            }
        }
    }
}





#[derive(Debug, Deserialize, Clone)]
pub struct Issue {
    pub title: String,
    pub html_url: String,
    /// task/ issue id referencing the foreign system
    pub id: String,

    #[serde(rename = "labels")]
    pub tags: Vec<Label>,
}


#[derive(Debug, Deserialize, Clone)]
pub struct Label {
    pub name: String,
}
