use reqwest::{
    header::{HeaderMap, ACCEPT, AUTHORIZATION, USER_AGENT},
    Client,
};

use crate::providers::common::model::Issue;
use crate::providers::github::model::GitHubConfig;
use crate::providers::{common::model::Label, github::model::GitHubIssue};
use crate::providers::github::SHORT_CODE_GITHUB;

use serde_json::json;

pub fn construct_github_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "User".parse().unwrap());
    headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
    headers.insert(ACCEPT, "application/vnd.github.v3+json".parse().unwrap());
    return headers;
}

pub async fn collect_tasks_from_github(
    github_config: &GitHubConfig,
) -> Result<Vec<Issue>, anyhow::Error> {
    let client = Client::new();
    let mut all_issues = Vec::new(); // Create a vector to collect all issues

    for (idx, repo) in github_config.repositories.iter().enumerate() {
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues",
            repo.owner, repo.repo
        );

        let response = client
            .get(&url)
            .headers(construct_github_header(&github_config.token))
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;

            let github_issues: Vec<GitHubIssue> = serde_json::from_str(&body)?;
            let issues = github_issues.into_iter().map(|github_issue| Issue {
                id: format!("{}{}/{}", SHORT_CODE_GITHUB, idx, github_issue.number),
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

pub async fn add_new_task(
    github_config: &GitHubConfig,
    title: &str,
    details: &str,
    tags: &Option<Vec<String>>,
) -> Result<(), anyhow::Error> {
    let client = Client::new();

    let default_repo = github_config
        .repositories
        .iter()
        .find(|repo| repo.default.unwrap_or(false))
        .expect("No default repository found");

    let add_url = format!(
        "https://api.github.com/repos/{}/{}/issues",
        default_repo.owner, default_repo.repo
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
        .headers(construct_github_header(&github_config.token))
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
