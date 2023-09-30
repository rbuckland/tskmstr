use anyhow::Result;
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::collections::HashSet;


use crate::config::{AppConfig, TaskIssueProvider};
use crate::providers::common::credentials::HasSecretToken;
use crate::providers::github::methods::{
    add_labels_to_github_issue, add_new_task_github, construct_github_header,
    remove_labels_from_github_issue,
};
use crate::providers::gitlab::methods::{
    add_labels_to_gitlab_issue, add_new_task_gitlab, construct_gitlab_header,
    remove_labels_from_gitlab_issue,
};

use crate::providers::o365::methods::add_new_task_o365;

use reqwest::Client;

const ERRMSG_DEFAULT_PROVIDER: &str = "No default provider was found. Ensure you have {defaults.for_newtasks: true} for your chosen provider";

/// add a new task is either
/// add a new task to the default todo provider
/// or, a provider_id is supplied (gl3, o365_2)
/// and that is used to locate the correct provider.
pub async fn add_new_task(
    provider_id: &Option<String>,
    config: &AppConfig,
    title: &str,
    details: &str,
    tags: &Option<Vec<String>>,
) -> Result<(), anyhow::Error> {
    debug!("creating new task {} {:?}", &title, &tags);
    debug!("default provider is {:?}", &config.find_default_provider());

    let x = match provider_id {
        None => config.find_default_provider().expect(ERRMSG_DEFAULT_PROVIDER).unwrap_or_else(||panic!("{}",ERRMSG_DEFAULT_PROVIDER)),
        Some(provider) => config.find_provider_for_issue(provider)
            .expect("The todo provider id was invalid").unwrap()
    };

    // Add the logic to call the appropriate add_new_task function based on the provider
    match x {
        TaskIssueProvider::GitHub(repo) => {
            add_new_task_github(
                &repo,
                &config.github_com.as_ref().unwrap(),
                &title,
                details,
                tags,
            )
            .await?
        }
        TaskIssueProvider::GitLab(repo) => {
            add_new_task_gitlab(
                &repo,
                &config.gitlab_com.as_ref().unwrap(),
                &title,
                details,
                tags,
            )
            .await?
        }
        TaskIssueProvider::O365(repo) => {
            add_new_task_o365(&repo, &config.o365.as_ref().unwrap(), &title, details, tags).await?
        }
        _ => return Err(anyhow::anyhow!("Unsupported provider")),
    }

    Ok(())
}

pub async fn remove_tags_from_task(
    app_config: &AppConfig,
    provider_and_issue: String,
    tags: &HashSet<String>,
) -> Result<(), anyhow::Error> {
   
    let details: Vec<&str> = provider_and_issue.split('/').collect();
    let provider_id = details.get(0).unwrap_or_else(||panic!("Provider ID was invalid"));
    let issue_id = details.get(1).unwrap_or_else(||panic!("Issue ID was invalid")).to_string();
    let repository = &app_config.find_provider_by_id(&provider_id)?.unwrap_or_else(||panic!("Provider was not found"));

    match repository {
        TaskIssueProvider::GitHub(repo_config) => {
            remove_labels_from_github_issue(
                &repo_config,
                &app_config.github_com.as_ref().unwrap(),
                &issue_id,
                &tags,
            )
            .await?
        }
        TaskIssueProvider::GitLab(repo_config) => {
            remove_labels_from_gitlab_issue(
                &repo_config,
                &app_config.gitlab_com.as_ref().unwrap(),
                &issue_id,
                &tags,
            )
            .await?
        }
        _ => unimplemented!(
            "Adding tags to an issue for {} is not supported yet",
            &provider_and_issue.clone()
        ),
    }

    Ok(())
}

pub async fn add_tags_to_task(
    app_config: &AppConfig,
    provider_and_issue: String,
    tags: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    let details: Vec<&str> = provider_and_issue.split('/').collect();
    let provider_id = details.get(0).unwrap_or_else(||panic!("Provider ID was invalid"));
    let issue_id = details.get(1).unwrap_or_else(||panic!("Issue ID was invalid")).to_string();
    let repository = &app_config.find_provider_by_id(&provider_id)?.unwrap_or_else(||panic!("Provider was not found"));

    match repository {
        TaskIssueProvider::GitHub(repo_config) => {
            add_labels_to_github_issue(
                &repo_config,
                &app_config.github_com.as_ref().unwrap(),
                &issue_id,
                &tags,
            )
            .await?
        }
        TaskIssueProvider::GitLab(repo_config) => {
            add_labels_to_gitlab_issue(
                &repo_config,
                &app_config.gitlab_com.as_ref().unwrap(),
                &issue_id,
                &tags,
            )
            .await?
        }
        _ => unimplemented!(
            "Adding tags to an issue for {} is not supported yet",
            &provider_and_issue.clone()
        ),
    }

    Ok(())
}

pub async fn close_task( app_config: &AppConfig,
    provider_and_issue: String) -> Result<()> {
    let client = Client::new();

    let details: Vec<&str> = provider_and_issue.split('/').collect();
    let provider_id = details.get(0).unwrap_or_else(||panic!("Provider ID was invalid"));
    let issue_id = details.get(1).unwrap_or_else(||panic!("Issue ID was invalid")).to_string();
    let repository = &app_config.find_provider_by_id(&provider_id)?.unwrap_or_else(||panic!("Provider was not found"));

    match repository {
        TaskIssueProvider::GitHub(repo_config) => {
            let url = format!(
                "https://api.github.com/repos/{}/{}/issues/{}",
                repo_config.owner, repo_config.repo, &issue_id
            );
            debug!("github: will close {}", url);

            let response = client
                .patch(&url)
                .headers(construct_github_header(
                    &app_config.github_com.as_ref().unwrap().get_token(),
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

        TaskIssueProvider::GitLab(repo_config) => {
            let url = format!(
                "https://gitlab.com/api/v4/projects/{}/issues/{}?state_event=close",
                repo_config.project_id, &issue_id
            );
            debug!("gitlab: will close {}", url);

            let response = client
                .put(&url)
                .headers(construct_gitlab_header(
                    &app_config.gitlab_com.as_ref().unwrap().get_token(),
                ))
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
        TaskIssueProvider::O365(_todolist_config) => {
            unimplemented!("o365 close not yet")
        }
    }

    Ok(())
}
