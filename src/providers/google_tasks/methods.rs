use log::{debug};
use std::collections::HashSet;

use reqwest::{
    header::{HeaderMap, ACCEPT, AUTHORIZATION, USER_AGENT},
    Client,
};

use crate::providers::common::{credentials::HasSecretToken, model::Issue};
use crate::providers::google_tasks::model::GoogleTaskConfig;
use crate::providers::{common::model::Label, google_tasks::model::GoogleTask};

use serde_json::json;

use super::model::GoogleTaskList;

pub fn construct_google_tasks_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "User".parse().unwrap());
    headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
    headers.insert(ACCEPT, "application/vnd.google_tasks.v3+json".parse().unwrap());
    headers
}

pub async fn close_task_google_tasks(
    google_tasks_config: &GoogleTaskConfig,
    task_list: &GoogleTaskList,
    issue_id: &String,
) -> Result<(), anyhow::Error> {
    // let client = Client::new();

    // let url = format!(
    //     "{}//tasks/v1/users/@me/lists/{}",
    //     google_tasks_config.endpoint, task_list.owner, task_list.task_list, &issue_id
    // );
    // debug!("google_tasks: will close {}", url);

    // let response = client
    //     .patch(&url)
    //     .headers(construct_google_tasks_header(&google_tasks_config.get_token()))
    //     .json(&serde_json::json!({
    //         "state": "closed"
    //     }))
    //     .send()
    //     .await?;

    // if response.status().is_success() {
    //     println!(
    //         "Task {} closed in GoogleTask task_list: {}/{}",
    //         issue_id, task_list.owner, task_list.task_list
    //     );
    // } else {
    //     println!(
    //         "Error: Unable to close task {} in GoogleTask task_list {}/{}. Status: {:?}",
    //         issue_id,
    //         task_list.owner,
    //         task_list.task_list,
    //         response.status()
    //     );
    // }

    Ok(())
}

pub async fn collect_tasks_from_google_tasks(
    google_tasks_config: &GoogleTaskConfig,
    provider_id: &Option<String>,
) -> Result<Vec<Issue>, anyhow::Error> {
    let client = Client::new();
    let mut all_issues = Vec::new(); // Create a vector to collect all issues

    for (_idx, task_list) in google_tasks_config
        .tasklists
        .iter()
        .filter(|&r| provider_id.is_none() || provider_id.as_deref().is_some_and(|p| r.id == p))
        .enumerate()
    {
        let url = format!(
            "{}//tasks/v1/users/@me/lists/{}",
            google_tasks_config.endpoint, task_list.id
        );

        let response = client
            .get(&url)
            .headers(construct_google_tasks_header(&google_tasks_config.get_token()))
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;

            let google_tasks_issues: Vec<GoogleTask> = serde_json::from_str(&body)?;
            let issues = google_tasks_issues.into_iter().map(|google_tasks_issue| Issue {
                id: format!("{}/{}", task_list.id, google_tasks_issue.id),
                title: google_tasks_issue.title,
                html_url: google_tasks_issue.url,
                tags: Vec::new() // google tasks have no labels
            });
            all_issues.extend(issues); // Add the collected issues to the vector
        } else {
            println!(
                "Error: Unable to fetch issues for {}/{}. Status: {:?}",
                task_list.id,
                task_list.title,
                response.status()
            );
        }
    }
    Ok(all_issues) // Return the vector of collected issues
}

pub async fn add_new_task_google_tasks(
    google_tasks_list: &GoogleTaskList,
    google_tasks_config: &GoogleTaskConfig,
    title: &str,
    details: &str,
    tags: &Option<Vec<String>>,
) -> Result<(), anyhow::Error> {
    // let client = Client::new();

    // let add_url = format!(
    //     "{}/task_lists/{}/{}/issues",
    //     google_tasks_config.endpoint, google_tasks_list.owner, google_tasks_list.task_list
    // );

    // let mut issue_details = json!({
    //     "title": title,
    //     "body": details,
    // });

    // if let Some(ts) = tags {
    //     issue_details["labels"] = serde_json::Value::Array(
    //         ts.iter()
    //             .map(|label| serde_json::Value::String(label.clone()))
    //             .collect(),
    //     );
    // }

    // let response = client
    //     .post(&add_url)
    //     .headers(construct_google_tasks_header(&google_tasks_config.get_token()))
    //     .header(reqwest::header::CONTENT_TYPE, "application/json")
    //     .body(issue_details.to_string())
    //     .send()
    //     .await?;

    // if response.status().is_success() {
    //     // debug!("{}",response.status());
    //     // let t = response.text().await?;
    //     let issue: GoogleTaskIssue = response.json::<GoogleTaskIssue>().await?;
    //     println!("New issue created:");
    //     println!("Title: {}", issue.title);
    //     println!("URL: {}", issue.html_url);
    // } else {
    //     println!(
    //         "Error: Unable to create issue. Status: {:?}",
    //         response.status()
    //     );
    // }
    Ok(())
}

use anyhow::{anyhow, Result};

pub async fn add_labels_to_google_tasks_issue(
    google_tasks_list: &GoogleTaskList,
    google_tasks_config: &GoogleTaskConfig,
    issue_number: &String,
    labels: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    // let client = Client::new();

    // // Create a URL for the GoogleTask API endpoint to add labels
    // let url = format!(
    //     "{}/task_lists/{}/{}/issues/{}/labels",
    //     google_tasks_config.endpoint, google_tasks_list.owner, google_tasks_list.task_list, issue_number
    // );

    // // Prepare the list of labels to add as JSON
    // let labels_json: Vec<String> = labels.iter().cloned().collect();
    // let json_body = json!(&labels_json);

    // // Send a POST request to add labels
    // let response = client
    //     .post(&url)
    //     .headers(construct_google_tasks_header(&google_tasks_config.get_token()))
    //     .header(reqwest::header::ACCEPT, "application/vnd.google_tasks.v3+json")
    //     .json(&json_body)
    //     .send()
    //     .await?;

    // // Check the response status and handle errors
    // if response.status().is_success() {
    //     Ok(())
    // } else {
    //     let error_msg = format!(
    //         "Failed to add labels to GoogleTask issue: {:?}",
    //         response.status()
    //     );
    //     Err(anyhow!(error_msg))
    // }

    Ok(())
}

pub async fn remove_labels_from_google_tasks_issue(
    google_tasks_list: &GoogleTaskList,
    google_tasks_config: &GoogleTaskConfig,
    issue_number: &String,
    labels: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    // let client = Client::new();

    // // Create a URL for the GoogleTask API endpoint to remove labels for a specific issue
    // let url = format!(
    //     "{}/task_lists/{}/{}/issues/{}/labels",
    //     google_tasks_config.endpoint, google_tasks_list.owner, google_tasks_list.task_list, issue_number
    // );

    // // Iterate through the labels and send DELETE requests for each label
    // for label in labels {
    //     let label_url = format!("{}/{}", url, label);

    //     // Send a DELETE request for the specific label
    //     let response = client
    //         .delete(&label_url)
    //         .headers(construct_google_tasks_header(&google_tasks_config.get_token()))
    //         .header(reqwest::header::ACCEPT, "application/vnd.google_tasks.v3+json")
    //         .send()
    //         .await?;

    //     // Check the response status for each label and handle errors
    //     if !response.status().is_success() {
    //         let error_msg = format!(
    //             "Failed to remove label '{}' from GoogleTask issue: {:?}",
    //             label,
    //             response.status()
    //         );
    //         return Err(anyhow!(error_msg));
    //     }
    // }

    Ok(())
}
