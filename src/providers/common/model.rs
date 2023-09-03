use std::fmt;

use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;

use crate::config::{AppConfig, TodoSupplier};
use crate::providers::gitlab::model::GitLabRepository;
use crate::providers::github::model::GitHubRepository;

use crate::providers::gitlab::SHORT_CODE_GITLAB;
use crate::providers::github::SHORT_CODE_GITHUB;
use crate::providers::o365::SHORT_CODE_O365;



/// a representation of gl0/555, gh7/1234 o365_1/333
/// in the form of 
/// GitHub(7,Some("1234"))
/// 
#[derive(Debug, Deserialize, Clone)]
pub enum TodoSource {
    GitHub(u16, Option<String>),
    GitLab(u16, Option<String>),
    O365(u16, Option<String>),
}

impl fmt::Display for TodoSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TodoSource::GitHub(todosrc_idx, x) => {
                write!(f, "{}{}/{:?}", SHORT_CODE_GITLAB, todosrc_idx, x)
            }
            TodoSource::GitLab(todosrc_idx, x) => {
                write!(f, "{}{}/{:?}", SHORT_CODE_GITHUB, todosrc_idx, x)
            }
            TodoSource::O365(todosrc_idx, x) => {
                write!(f, "{}{}/{:?}", SHORT_CODE_O365, todosrc_idx, x)
            }
        }
    }
}

lazy_static! {
    /// Regex for tskmstr representations of a task across all task providers
    pub static ref TASK_ID_RE: Regex = Regex::new(format!(r"^(?<task_source>{}|{}|{})(?<todosrc_idx>[0-9]+)(/(?<issue_id>[A-Za-z0-9_-]+))?$",SHORT_CODE_GITHUB, SHORT_CODE_GITLAB, SHORT_CODE_O365).as_str()).unwrap();
}

impl std::str::FromStr for TodoSource {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = TASK_ID_RE.captures(s) {
            let task_source = captures.name("task_source").unwrap().as_str();
            let todosrc_idx: u16 = captures.name("todosrc_idx").unwrap().as_str().parse().unwrap();
            let issue_id = captures.name("issue_id").map(|m| m.as_str().to_string());

            
            match task_source {
                SHORT_CODE_GITHUB => Ok(TodoSource::GitHub(todosrc_idx, issue_id)),
                SHORT_CODE_GITLAB => Ok(TodoSource::GitLab(todosrc_idx, issue_id)),
                SHORT_CODE_O365 => Ok(TodoSource::O365(todosrc_idx, issue_id)),
                _ => Err(anyhow::anyhow!("Invalid task source")),
            }
        } else {
            Err(anyhow::anyhow!("Invalid task ID format"))
        }
    }
}

impl TodoSource {
    /// given we have been pre-seeded from from_str("gl0/222") return the appropriate config
    /// for the chosen, indexed repo/todo list provider
    pub fn task_supplier<'a>(
        self,
        app_config: &'a AppConfig,
    ) -> TodoSupplier {
        match self {
            TodoSource::GitHub(todosrc_idx, _) => {
                let github_config = &app_config.github_com;
                let github_repo = github_config
                    .as_ref()
                    .and_then(|config| config.repositories.get(todosrc_idx as usize));
                TodoSupplier::GitHub(github_repo.expect("Invalud github index").clone())
            }
            TodoSource::GitLab(todosrc_idx, _) => {
                let gitlab_config = &app_config.gitlab_com;
                let gitlab_repo = gitlab_config
                    .as_ref()
                    .and_then(|config| config.repositories.get(todosrc_idx as usize));
                TodoSupplier::GitLab(gitlab_repo.expect("Invalud gitlab index").clone())
            }
            TodoSource::O365(todosrc_idx, _) => {
                let o365_config = &app_config.o365;
                let o365_repo = o365_config
                    .as_ref()
                    .and_then(|config| config.todo_lists.get(todosrc_idx as usize));
                TodoSupplier::O365(o365_repo.expect("Invalud o365 index").clone())
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
