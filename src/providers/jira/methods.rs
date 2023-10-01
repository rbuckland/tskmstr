use std::collections::HashSet;

use anyhow::{Result};
use reqwest::{Client, header::HeaderMap};

use base64::{Engine as _, engine::{self, general_purpose}, alphabet};

use crate::providers::{common::credentials::HasSecretToken, jira::model::JiraResult};
use crate::providers::common::model::{Issue, Label};
use crate::providers::jira::model::JiraConfig;
use log::{debug, error, info, warn};

use super::model::{JiraProject, JiraIssue};

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
                .headers(construct_jira_basic_auth_header(&j.get_username(), &j.get_token()))
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
                    title: jira_issue.fields.summary,
                    html_url: jira_issue.url,
                    tags: jira_issue.fields
                        .labels.unwrap_or(Vec::new())
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
    jira_repo: &JiraProject,
    jira_config: &JiraConfig,
    title: &str,
    details: &str,
    tags: &Option<Vec<String>>,
) -> Result<(), anyhow::Error> {
    // Implement the method to add a new task in Jira
    // Use the Jira API to create a new issue
    unimplemented!("not complete")
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
