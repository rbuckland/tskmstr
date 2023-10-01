use std::collections::HashSet;

use anyhow::{anyhow, Result};
use reqwest::{header::HeaderMap, Client};
use serde_json::json;

use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};

use crate::providers::{common::model::{Issue, Label}, jira::model::JiraIssueType};
use crate::providers::{common::credentials::HasSecretToken, jira::model::JiraResult};
use log::{debug, error, info, warn};
use super::model::{JiraIssue, JiraProject, JiraConfig};

pub fn construct_jira_basic_auth_header(username: &str, token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let b64 = general_purpose::STANDARD.encode(format!("{}:{}", username, token));
    let auth_value = format!("Basic {}", b64);
    headers.insert("Authorization", auth_value.parse().unwrap());
    headers
}

pub async fn collect_tasks_from_jira(
    jira_config: &Vec<JiraConfig>,
    provider_id: &Option<String>,
) -> Result<Vec<Issue>, anyhow::Error> {
    let client = Client::new();
    let mut all_issues = Vec::new();

    for j in jira_config {
        for (_idx, project) in j
            .projects
            .iter()
            .filter(|&r| provider_id.is_none() || provider_id.as_deref().is_some_and(|p| r.id == p))
            .enumerate()
        {
            // Construct the Jira API URL for fetching issues
            let url = format!(
                "{}/rest/api/3/search?jql=project={}&maxResults=1000", //%20AND%20status=Open&maxResults=1000
                j.endpoint, project.project_key
            );

            debug!("{}", url);

            // Send a GET request to fetch issues
            let response = client
                .get(&url)
                .headers(construct_jira_basic_auth_header(
                    &j.get_username(),
                    &j.get_token(),
                ))
                .send()
                .await?;

            if response.status().is_success() {
                let body = response.text().await?;

                debug!("{}", body);

                // Deserialize Jira issues into your internal Issue representation
                let j_result: JiraResult = serde_json::from_str(&body)?;
                // Convert Jira issues to the internal Issue representation
                let issues = j_result.issues.into_iter().map(|jira_issue| Issue {
                    id: format!("{}/{}", project.id, jira_issue.id),
                    title: jira_issue.fields.as_ref().unwrap().summary.clone(),
                    html_url: jira_issue.url,
                    tags: jira_issue
                        .fields.unwrap()
                        .labels
                        .unwrap_or(Vec::new())
                        .into_iter()
                        .map(|label| Label { name: label })
                        .collect(),
                });

                all_issues.extend(issues);
            } else {
                println!(
                    "Error: Unable to fetch issues for project_id {}. Status: {:?}",
                    project.project_key,
                    response.status()
                );
            }
        }
    }

    Ok(all_issues)
}

pub async fn add_new_task_jira(
    jira_project: &JiraProject,
    jira_config: &JiraConfig,
    title: &str,
    details: &str,
    tags: &Option<Vec<String>>,
) -> Result<(), anyhow::Error> {
    let client = Client::new();

    let add_url = format!("{}/rest/api/2/issue/", jira_config.endpoint);

    // TODO we can just use the struct instead of -partial" creation. FIXME
    let mut issue_details = json!({
        "fields": {
            "project": {
                "key": jira_project.project_key
            },
            "summary": title,
            "description": details,
            "issuetype" : JiraIssueType { name: jira_project.default_issue_type.clone() },
        }
    });

    if let Some(ts) = tags {
        issue_details["fields"]["labels"] = serde_json::Value::Array(
            ts.iter()
                .map(|label| serde_json::Value::String(label.clone()))
                .collect(),
        );
    }

    debug!("posting {} to {}", issue_details, add_url);

    let response = client
        .post(&add_url)
        .headers(construct_jira_basic_auth_header(
            &jira_config.get_username(),
            &jira_config.get_token(),
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&issue_details)
        .send()
        .await?;

    if response.status().is_success() {
        let issue: JiraIssue = response.json::<JiraIssue>().await?;
        println!("New issue created:");
        println!("ID: {}", issue.id);
        println!("URL: {}", issue.url);
    } else {
        println!(
            "Error: Unable to create issue. Status: {:?}",
            response.status()
        );
    }
    Ok(())
}

pub async fn add_labels_to_jira_issue(
    jira_repo: &JiraProject,
    jira_config: &JiraConfig,
    issue_id: &String,
    tags: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    // Implement the method to add labels to a Jira issue
    // Use the Jira API to update the issue's labels
    unimplemented!("not complete")
}

pub async fn remove_labels_from_jira_issue(
    jira_repo: &JiraProject,
    jira_config: &JiraConfig,
    issue_id: &String,
    tags: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    // Implement the method to remove labels from a Jira issue
    // Use the Jira API to update the issue's labels
    unimplemented!("not complete")
}
