use log::{debug};
use std::collections::HashSet;

use reqwest::{
    header::{HeaderMap, ACCEPT, AUTHORIZATION, USER_AGENT},
    Client,
};

use crate::providers::common::{credentials::HasSecretToken, model::Issue};
use crate::providers::github::model::GitHubConfig;
use crate::providers::{common::model::Label, github::model::GitHubIssue};

use serde_json::json;

use super::model::GitHubRepository;

pub fn construct_github_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "User".parse().unwrap());
    headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
    headers.insert(ACCEPT, "application/vnd.github.v3+json".parse().unwrap());
    headers
}

pub async fn close_task_github(
    github_config: &GitHubConfig,
    repo_config: &GitHubRepository,
    issue_id: &String,
) -> Result<(), anyhow::Error> {
    let client = Client::new();

    let url = format!(
        "{}/repos/{}/{}/issues/{}",
        github_config.endpoint, repo_config.owner, repo_config.repo, &issue_id
    );
    debug!("github: will close {}", url);

    let response = client
        .patch(&url)
        .headers(construct_github_header(&github_config.get_token()))
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

    Ok(())
}

pub async fn collect_tasks_from_github(
    github_config: &GitHubConfig,
    provider_id: &Option<String>,
) -> Result<Vec<Issue>, anyhow::Error> {
    let client = Client::new();
    let mut all_issues = Vec::new(); // Create a vector to collect all issues

    for (_idx, repo) in github_config
        .repositories
        .iter()
        .filter(|&r| provider_id.is_none() || provider_id.as_deref().is_some_and(|p| r.id == p))
        .enumerate()
    {
        let url = format!(
            "{}/repos/{}/{}/issues",
            github_config.endpoint, repo.owner, repo.repo
        );

        let response = client
            .get(&url)
            .headers(construct_github_header(&github_config.get_token()))
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;

            let github_issues: Vec<GitHubIssue> = serde_json::from_str(&body)?;
            let issues = github_issues.into_iter().map(|github_issue| Issue {
                id: format!("{}/{}", repo.id, github_issue.number),
                title: github_issue.title,
                html_url: github_issue.html_url,
                tags: github_issue
                    .labels
                    .into_iter()
                    .map(|l| Label { name: l.name })
                    .collect(),
            });
            all_issues.extend(issues); // Add the collected issues to the vector
        } else {
            println!(
                "Error: Unable to fetch issues for {}/{}. Status: {:?}",
                repo.owner,
                repo.repo,
                response.status()
            );
        }
    }
    Ok(all_issues) // Return the vector of collected issues
}

pub async fn add_new_task_github(
    github_repo: &GitHubRepository,
    github_config: &GitHubConfig,
    title: &str,
    details: &str,
    tags: &Option<Vec<String>>,
) -> Result<(), anyhow::Error> {
    let client = Client::new();

    let add_url = format!(
        "{}/repos/{}/{}/issues",
        github_config.endpoint, github_repo.owner, github_repo.repo
    );

    let mut issue_details = json!({
        "title": title,
        "body": details,
    });

    if let Some(ts) = tags {
        issue_details["labels"] = serde_json::Value::Array(
            ts.iter()
                .map(|label| serde_json::Value::String(label.clone()))
                .collect(),
        );
    }

    let response = client
        .post(&add_url)
        .headers(construct_github_header(&github_config.get_token()))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(issue_details.to_string())
        .send()
        .await?;

    if response.status().is_success() {
        // debug!("{}",response.status());
        // let t = response.text().await?;
        let issue: GitHubIssue = response.json::<GitHubIssue>().await?;
        println!("New issue created:");
        println!("Title: {}", issue.title);
        println!("URL: {}", issue.html_url);
    } else {
        println!(
            "Error: Unable to create issue. Status: {:?}",
            response.status()
        );
    }
    Ok(())
}

use anyhow::{anyhow, Result};

pub async fn add_labels_to_github_issue(
    github_repo: &GitHubRepository,
    github_config: &GitHubConfig,
    issue_number: &String,
    labels: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    let client = Client::new();

    // Create a URL for the GitHub API endpoint to add labels
    let url = format!(
        "{}/repos/{}/{}/issues/{}/labels",
        github_config.endpoint, github_repo.owner, github_repo.repo, issue_number
    );

    // Prepare the list of labels to add as JSON
    let labels_json: Vec<String> = labels.iter().cloned().collect();
    let json_body = json!(&labels_json);

    // Send a POST request to add labels
    let response = client
        .post(&url)
        .headers(construct_github_header(&github_config.get_token()))
        .header(reqwest::header::ACCEPT, "application/vnd.github.v3+json")
        .json(&json_body)
        .send()
        .await?;

    // Check the response status and handle errors
    if response.status().is_success() {
        Ok(())
    } else {
        let error_msg = format!(
            "Failed to add labels to GitHub issue: {:?}",
            response.status()
        );
        Err(anyhow!(error_msg))
    }
}

pub async fn remove_labels_from_github_issue(
    github_repo: &GitHubRepository,
    github_config: &GitHubConfig,
    issue_number: &String,
    labels: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    let client = Client::new();

    // Create a URL for the GitHub API endpoint to remove labels for a specific issue
    let url = format!(
        "{}/repos/{}/{}/issues/{}/labels",
        github_config.endpoint, github_repo.owner, github_repo.repo, issue_number
    );

    // Iterate through the labels and send DELETE requests for each label
    for label in labels {
        let label_url = format!("{}/{}", url, label);

        // Send a DELETE request for the specific label
        let response = client
            .delete(&label_url)
            .headers(construct_github_header(&github_config.get_token()))
            .header(reqwest::header::ACCEPT, "application/vnd.github.v3+json")
            .send()
            .await?;

        // Check the response status for each label and handle errors
        if !response.status().is_success() {
            let error_msg = format!(
                "Failed to remove label '{}' from GitHub issue: {:?}",
                label,
                response.status()
            );
            return Err(anyhow!(error_msg));
        }
    }

    Ok(())
}
