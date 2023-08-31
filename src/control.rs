use anyhow::Result;
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::str::FromStr;

use crate::config::{AppConfig, TodoSupplier};
use crate::providers::common::model::TodoSource;
use crate::providers::o365::SHORT_CODE_O365;
use crate::providers::github::SHORT_CODE_GITHUB;
use crate::providers::gitlab::SHORT_CODE_GITLAB;
use crate::providers::github::methods::{construct_github_header, add_new_task_github};
use crate::providers::gitlab::methods::{construct_gitlab_header, add_new_task_gitlab};
use crate::providers::o365::methods::add_new_task_o365;

use reqwest::Client;


/// add a new task is either
/// add a new task to the default todo provider
/// or, a provider_id is supplied (gl3, o365_2)
/// and that is used to locate the correct provider.
pub async fn add_new_task(
    provider_and_id: &Option<String>,
    config: &AppConfig,
    title: &str,
    details: &str,
    tags: &Option<Vec<String>>,
) -> Result<(), anyhow::Error> {


    debug!("creating new task {} {:?}", &title, &tags);
    
    let x = match provider_and_id { 
        None => config.default_todo_source().expect("failed").unwrap(),
        Some(p_and_id) => {
            TodoSource::from_str(&p_and_id).expect("the todo provider id was invalid").task_supplier(&config)
        }
    };

    // Add the logic to call the appropriate add_new_task function based on the provider
    match x {
        TodoSupplier::GitHub(repo) => add_new_task_github(&repo, &config.github_com.as_ref().unwrap(), &title, details, tags).await?,
        TodoSupplier::GitLab(repo) => add_new_task_gitlab(&repo, &config.gitlab_com.as_ref().unwrap(), &title, details, tags).await?,
        TodoSupplier::O365(repo) => add_new_task_o365(&repo, &config.o365.as_ref().unwrap(), &title, details, tags).await?,
        _ => return Err(anyhow::anyhow!("Unsupported provider")),
    }

    Ok(())
}



pub async fn close_task(source: TodoSource, app_config: &AppConfig) -> Result<()> {
    let client = Client::new();

    let issue_id = match &source {
        TodoSource::GitHub(_, id) => id.clone().unwrap(),
        TodoSource::GitLab(_, id) => id.clone().unwrap(),
        _ => unimplemented!("closing a task for {} is not supported yet", &source)
    };

    let repository = source.task_supplier(&app_config);

    match repository {
        TodoSupplier::GitHub(repo_config) => {

            let url = format!(
                "https://api.github.com/repos/{}/{}/issues/{}",
                repo_config.owner, repo_config.repo, &issue_id
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
            
        TodoSupplier::GitLab(repo_config) => {
            let url = format!(
                "https://gitlab.com/api/v4/projects/{}/issues/{}",
                repo_config.project_id, &issue_id
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
        TodoSupplier::O365(todolist_config) => {
            unimplemented!("o365 close not yet")
        }
        }

    Ok(())
}
