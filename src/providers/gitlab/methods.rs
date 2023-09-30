use std::collections::HashSet;

use crate::providers::common::credentials::HasSecretToken;
use crate::providers::common::model::Issue;
use crate::providers::gitlab::model::GitLabConfig;
use crate::providers::gitlab::model::GitLabIssue;
use crate::providers::gitlab::SHORT_CODE_GITLAB;
use anyhow::{Result, anyhow};
use serde_json::json;

use crate::providers::common::model::Label;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use reqwest::{header::HeaderMap, Client};

use super::model::GitLabRepository;

pub fn construct_gitlab_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("PRIVATE-TOKEN", format!("{}", token).parse().unwrap());
    return headers;
}

pub async fn collect_tasks_from_gitlab(
    gitlab_config: &GitLabConfig,
    provider_id: &Option<String>,
) -> Result<Vec<Issue>, anyhow::Error> {
    let client = Client::new();
    let mut all_issues = Vec::new();

    for (idx, repo) in gitlab_config
    .repositories
    .iter()
    .filter(|&r| provider_id.is_none() || provider_id.as_deref().is_some_and(|p| r.id == p))
    .enumerate()
{

        let url = format!(
            "https://gitlab.com/api/v4/projects/{}/issues?state=opened",
            repo.project_id
        );

        let response = client
            .get(&url)
            .header("PRIVATE-TOKEN", gitlab_config.get_token())
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;
            debug!("{}", body);

            let gitlab_issues: Vec<GitLabIssue> = serde_json::from_str(&body)?;
            // Convert GitLab issues to the internal Issue representation
            let issues = gitlab_issues.into_iter().map(|gitlab_issue| Issue {
                id: format!("{}/{}", repo.id, gitlab_issue.iid),
                title: gitlab_issue.title,
                html_url: gitlab_issue.web_url,
                tags: gitlab_issue
                    .labels
                    .into_iter()
                    .map(|label| Label { name: label.0 })
                    .collect(),
            });

            all_issues.extend(issues);
        } else {
            println!(
                "Error: Unable to fetch issues for project_id {}. Status: {:?}",
                repo.project_id,
                response.status()
            );
        }
    }

    Ok(all_issues)
}

pub async fn add_new_task_gitlab(
    gitlab_repo: &GitLabRepository,
    gitlab_config: &GitLabConfig,
    title: &str,
    details: &str,
    tags: &Option<Vec<String>>,
) -> Result<(), anyhow::Error> {

    debug!("Adding a new task via gitlab: {} [{:?}]", &title, &tags);

    let client = Client::new();

     let add_url = format!(
        "https://gitlab.com/api/v4/projects/{}/issues",
        gitlab_repo.project_id
    );

    let mut issue_details = json!({
        "title": title,
        "description": details,
    });

    if let Some(ts) = tags {
        issue_details["labels"] = ts.iter().map(|label| label.clone()).collect::<Vec<String>>().into();
    }

    let response = client
        .post(&add_url)
        .headers(construct_gitlab_header(&gitlab_config.get_token()))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&issue_details)
        .send()
        .await?;

    if response.status().is_success() {
        // debug!("{}",response.status());
        // let t = response.text().await?;
        let issue: GitLabIssue = response.json::<GitLabIssue>().await?;
        println!("New issue created:");
        println!("Title: {}", issue.title);
        println!("URL: {}", issue.web_url);
    } else {
        println!(
            "Error: Unable to create issue. Status: {:?}",
            response.status()
        );
    }
    Ok(())
}

pub async fn add_labels_to_gitlab_issue(
    gitlab_repo: &GitLabRepository,
    gitlab_config: &GitLabConfig,
    issue_id: &String, tags: &HashSet<String>
) -> Result<(), anyhow::Error> {

    let client = Client::new();

    // Create a URL for the GitLab API endpoint to add labels
    let url = format!(
        "https://gitlab.com/api/v4/projects/{}/issues/{}/add_labels",
        gitlab_repo.project_id,
        issue_id
    );

    // Prepare the list of labels to add as JSON
    let labels: Vec<&str> = tags.iter().map(|tag| tag.as_str()).collect();
    let json_body = json!({
        "labels": labels
    });

    // Send a POST request to add labels
    let response = client
        .post(&url)
        .headers(construct_gitlab_header(&gitlab_config.get_token()))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&json_body)
        .send().await?;

    // Check the response status and handle errors
    if response.status().is_success() {
        Ok(())
    } else {
        let error_msg = format!("Failed to add labels to GitLab issue: {:?}", response.status());
        Err(anyhow!(error_msg))
    }
}

pub async fn remove_labels_from_gitlab_issue(
    gitlab_repo: &GitLabRepository,
    gitlab_config: &GitLabConfig,
    issue_id: &String, tags: &HashSet<String>
) -> Result<(), anyhow::Error> {

    let client = Client::new();

    // Create a URL for the GitLab API endpoint to add labels
    let url = format!(
        "https://gitlab.com/api/v4/projects/{}/issues/{}/remove_labels",
        gitlab_repo.project_id,
        issue_id
    );

    // Prepare the list of labels to add as JSON
    let labels: Vec<&str> = tags.iter().map(|tag| tag.as_str()).collect();
    let json_body = json!({
        "labels": labels
    });

    // Send a POST request to add labels
    let response = client
        .post(&url)
        .headers(construct_gitlab_header(&gitlab_config.get_token()))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&json_body)
        .send().await?;

    // Check the response status and handle errors
    if response.status().is_success() {
        Ok(())
    } else {
        let error_msg = format!("Failed to remove labels from GitLab issue: {:?}", response.status());
        Err(anyhow!(error_msg))
    }
}