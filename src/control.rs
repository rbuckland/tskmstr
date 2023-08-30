use anyhow::Result;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use crate::config::AppConfig;
use crate::providers::common::model::TodoSource;
use crate::providers::github::methods::construct_github_header;
use crate::providers::gitlab::methods::construct_gitlab_header;

use reqwest::Client;

pub async fn close_task(source: TodoSource, app_config: &AppConfig) -> Result<()> {
    let client = Client::new();

    let issue_id = match &source {
        TodoSource::GitHub(_, id) => id.clone(),
        TodoSource::GitLab(_, id) => id.clone(),
    };

    match source {
        TodoSource::GitHub(_, _) => {
            let repo_config = source.task_supplier(&app_config).left().unwrap();
            let url = format!(
                "https://api.github.com/repos/{}/{}/issues/{}",
                repo_config.owner, repo_config.repo, issue_id
            );
            debug!("github: will close {}", url);

            let response = client
                .patch(&url)
                .headers(construct_github_header(
                    &app_config.github_com.as_ref().unwrap().token,
                ))
                .json(&serde_json::json!({
                    "state": "closed"
                }))
                .send()
                .await?;

            if response.status().is_success() {
                println!(
                    "Task {} closed in GitHub repo: {}/{}",
                    issue_id, repo_config.owner, repo_config.repo
                );
            } else {
                println!(
                    "Error: Unable to close task {} in GitHub repo {}/{}. Status: {:?}",
                    issue_id,
                    repo_config.owner,
                    repo_config.repo,
                    response.status()
                );
            }
        }
        TodoSource::GitLab(_, _) => {
            let repo_config = source.task_supplier(&app_config).right().unwrap();

            let url = format!(
                "https://gitlab.com/api/v4/projects/{}/issues/{}",
                repo_config.project_id, issue_id
            );
            debug!("gitlab: will close {}", url);

            let response = client
                .put(&url)
                .headers(construct_gitlab_header(
                    &app_config.gitlab_com.as_ref().unwrap().token,
                ))
                .json(&serde_json::json!({
                    "state_event": "close"
                }))
                .send()
                .await?;

            if response.status().is_success() {
                println!(
                    "Task {} closed in GitLab project: {}",
                    issue_id, repo_config.project_id
                );
            } else {
                println!(
                    "Error: Unable to close {} issue in GitLab project: {}. Status: {:?}",
                    issue_id,
                    repo_config.project_id,
                    response.status()
                );
            }
        }
    }

    Ok(())
}
