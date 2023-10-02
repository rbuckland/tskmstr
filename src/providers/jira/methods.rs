use std::collections::{HashMap, HashSet};

use anyhow::Result;
use reqwest::{header::HeaderMap, Client};
use serde_json::json;

use base64::{engine::general_purpose, Engine as _};

use super::model::{JiraConfig, JiraIssue, JiraProject};
use crate::providers::{common::credentials::HasSecretToken, jira::model::JiraResult};
use crate::providers::{
    common::model::{Issue, Label},
    jira::model::JiraIssueType,
};
use log::debug;

pub fn construct_jira_basic_auth_header(username: &str, token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let b64 = general_purpose::STANDARD.encode(format!("{}:{}", username, token));
    let auth_value = format!("Basic {}", b64);
    headers.insert("Authorization", auth_value.parse().unwrap());
    headers
}

pub async fn collect_tasks_from_jira(
    jira_config: &Vec<JiraConfig>,
    issue_store_id: &Option<String>,
) -> Result<Vec<Issue>, anyhow::Error> {
    let client = Client::new();
    let mut all_issues = Vec::new();

    for j in jira_config {
        for (_idx, project) in j
            .projects
            .iter()
            .filter(|&r| issue_store_id.is_none() || issue_store_id.as_deref().is_some_and(|p| r.id == p))
            .enumerate()
        {
            let optional_filter = project
                .filter
                .as_ref()
                .map_or("".to_string(), |filt| format!(" AND {}", filt));

            // Construct the Jira API URL for fetching issues
            let url = format!(
                "{}/rest/api/3/search?jql=project={} AND resolution = unresolved{}&maxResults=1000",
                j.endpoint, project.project_key, optional_filter
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
                        .fields
                        .unwrap()
                        .labels
                        .unwrap_or_default()
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

pub async fn remove_labels_from_jira_issue(
    jira_config: &JiraConfig,
    issue_id: &String,
    tags: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    let client = Client::new();

    // The Jira API endpoint for updating an issue
    let issue_url = format!("{}/rest/api/2/issue/{}", jira_config.endpoint, issue_id);

    let mut labels = serde_json::Map::new();
    labels.insert(
        "labels".to_string(),
        serde_json::Value::Array(
            tags.iter()
                .map(|label| json!({ "remove" : label.clone()}))
                .collect(),
        ),
    );

    let update = json!({
        "update": labels
    });

    debug!("posting {} to {}", update, issue_url);

    // Send a PUT request to update the issue's labels
    let response = client
        .put(&issue_url)
        .headers(construct_jira_basic_auth_header(
            &jira_config.get_username(),
            &jira_config.get_token(),
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&update)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Labels {:?} removed from issue {}.", tags, issue_id);
    } else {
        println!(
            "Error: Unable to add labels to issue {}. Status: {:?} Code: {:?}",
            issue_id,
            response.status(),
            response.text().await?,
        );
    }

    Ok(())
}

pub async fn add_labels_to_jira_issue(
    jira_config: &JiraConfig,
    issue_id: &String,
    tags: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    let client = Client::new();

    // The Jira API endpoint for updating an issue
    let issue_url = format!("{}/rest/api/2/issue/{}", jira_config.endpoint, issue_id);

    let mut labels = serde_json::Map::new();
    labels.insert(
        "labels".to_string(),
        serde_json::Value::Array(
            tags.iter()
                .map(|label| json!({ "add" : label.clone()}))
                .collect(),
        ),
    );

    let update = json!({
        "update": labels
    });

    debug!("posting {} to {}", update, issue_url);

    // Send a PUT request to update the issue's labels
    let response = client
        .put(&issue_url)
        .headers(construct_jira_basic_auth_header(
            &jira_config.get_username(),
            &jira_config.get_token(),
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&update)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Labels {:?} added to issue {}.", tags, issue_id);
    } else {
        println!(
            "Error: Unable to add labels to issue {}. Status: {:?} Code: {:?}",
            issue_id,
            response.status(),
            response.text().await?,
        );
    }

    Ok(())
}

pub async fn close_issue_jira(
    jira_config: &JiraConfig,
    project_config: &JiraProject,
    issue_id: &String,
) -> Result<(), anyhow::Error> {
    let client = Client::new();

    // The Jira API endpoint for transitioning issues
    let transition_url = format!(
        "{}/rest/api/3/issue/{}/transitions",
        jira_config.endpoint, issue_id
    );

    // The payload for the transition request
    let transition_payload = json!({
        "transition": {
            "id": project_config.close_transition_id,// this is a jira instance, project specific value
            // use tskmstr jira-transitions <issue-id> to locate it
        }
    });

    debug!("posting {} to {}", transition_payload, transition_url);

    let response = client
        .post(&transition_url)
        .headers(construct_jira_basic_auth_header(
            &jira_config.get_username(),
            &jira_config.get_token(),
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&transition_payload)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Issue {} has been closed.", issue_id);
    } else {
        println!(
            "Error: Unable to close issue {}. Do you have the transition_id correct ? transition_id={}, Status: {:?}",
            issue_id,
            project_config.close_transition_id,
            response.status()
        );
    }
    Ok(())
}

/// a utility method, to determine what Transition Id's to put in config
pub async fn list_jira_transition_ids(
    jira_config: &JiraConfig,
    issue_id: &String,
) -> Result<(), anyhow::Error> {
    let client = Client::new();

    // The Jira API endpoint for transitioning issues
    let transition_url = format!(
        "{}/rest/api/3/issue/{}/transitions",
        jira_config.endpoint, issue_id
    );

    let response = client
        .get(&transition_url)
        .headers(construct_jira_basic_auth_header(
            &jira_config.get_username(),
            &jira_config.get_token(),
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        let text = response.text().await?;
        let json_resp: HashMap<String, serde_json::Value> = serde_json::from_str(&text).unwrap();
        println!("{}", serde_json::to_string_pretty(&json_resp).unwrap());
    } else {
        println!(
            "Error: Getting the transitions for {}. Status: {:?}",
            issue_id,
            response.status()
        );
    }
    Ok(())
}
