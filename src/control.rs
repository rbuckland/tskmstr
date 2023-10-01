use anyhow::Result;
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::collections::HashSet;

use crate::config::{AppConfig, TaskIssueProvider};

use crate::providers::github::methods::{
    add_labels_to_github_issue, add_new_task_github, close_task_github,
    remove_labels_from_github_issue,
};
use crate::providers::gitlab::methods::{
    add_labels_to_gitlab_issue, add_new_task_gitlab, close_task_gitlab,
    remove_labels_from_gitlab_issue,
};
use crate::providers::jira::methods::{
    add_labels_to_jira_issue, add_new_task_jira, close_issue_jira, remove_labels_from_jira_issue,
};



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
        None => config
            .find_default_provider()
            .expect(ERRMSG_DEFAULT_PROVIDER)
            .unwrap_or_else(|| panic!("{}", ERRMSG_DEFAULT_PROVIDER)),
        Some(provider) => config
            .find_provider_for_issue(provider)
            .expect("The todo provider id was invalid")
            .unwrap(),
    };

    // Add the logic to call the appropriate add_new_task function based on the provider
    match x {
        TaskIssueProvider::GitHub(github_config, repo) => {
            add_new_task_github(&repo, &github_config, title, details, tags).await?
        }
        TaskIssueProvider::GitLab(gitlab_config, repo) => {
            add_new_task_gitlab(&repo, &gitlab_config, title, details, tags).await?
        }

        TaskIssueProvider::Jira(jira_config, project) => {
            add_new_task_jira(&project, &jira_config, title, details, tags).await?
        }
    }

    Ok(())
}

pub async fn remove_tags_from_task(
    app_config: &AppConfig,
    provider_and_issue: String,
    tags: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    let details: Vec<&str> = provider_and_issue.split('/').collect();
    let provider_id = details.first()
        .unwrap_or_else(|| panic!("Provider ID was invalid"));
    let issue_id = details
        .get(1)
        .unwrap_or_else(|| panic!("Issue ID was invalid"))
        .to_string();
    let repository = &app_config
        .find_provider_by_id(provider_id)?
        .unwrap_or_else(|| panic!("Provider was not found"));

    match repository {
        TaskIssueProvider::GitHub(github_config, repo_config) => {
            remove_labels_from_github_issue(repo_config, github_config, &issue_id, tags).await?
        }
        TaskIssueProvider::GitLab(gitlab_config, repo_config) => {
            remove_labels_from_gitlab_issue(repo_config, gitlab_config, &issue_id, tags).await?
        }
        TaskIssueProvider::Jira(jira_config, _) => {
            remove_labels_from_jira_issue(jira_config, &issue_id, tags).await?
        }
    }

    Ok(())
}

pub async fn add_tags_to_task(
    app_config: &AppConfig,
    provider_and_issue: String,
    tags: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    let details: Vec<&str> = provider_and_issue.split('/').collect();
    let provider_id = details.first()
        .unwrap_or_else(|| panic!("Provider ID was invalid"));
    let issue_id = details
        .get(1)
        .unwrap_or_else(|| panic!("Issue ID was invalid"))
        .to_string();
    let repository = &app_config
        .find_provider_by_id(provider_id)?
        .unwrap_or_else(|| panic!("Provider was not found"));

    match repository {
        TaskIssueProvider::GitHub(github_config, repo_config) => {
            add_labels_to_github_issue(repo_config, github_config, &issue_id, tags).await?
        }
        TaskIssueProvider::GitLab(gitlab_config, repo_config) => {
            add_labels_to_gitlab_issue(repo_config, gitlab_config, &issue_id, tags).await?
        }

        TaskIssueProvider::Jira(jira_config, _) => {
            add_labels_to_jira_issue(jira_config, &issue_id, tags).await?
        }
    }

    Ok(())
}

pub async fn close_task(app_config: &AppConfig, provider_and_issue: String) -> Result<()> {
    let details: Vec<&str> = provider_and_issue.split('/').collect();
    let provider_id = details.first()
        .unwrap_or_else(|| panic!("Provider ID was invalid"));
    let issue_id = details
        .get(1)
        .unwrap_or_else(|| panic!("Issue ID was invalid"))
        .to_string();
    let repository = &app_config
        .find_provider_by_id(provider_id)?
        .unwrap_or_else(|| panic!("Provider was not found"));

    match repository {
        TaskIssueProvider::GitHub(github_config, repo_config) => {
            close_task_github(github_config, repo_config, &issue_id).await?
        }
        TaskIssueProvider::GitLab(gitlab_config, repo_config) => {
            close_task_gitlab(gitlab_config, repo_config, &issue_id).await?
        }
        TaskIssueProvider::Jira(jira_config, project_config) => {
            close_issue_jira(jira_config, project_config, &issue_id).await?
        }
    }

    Ok(())
}
