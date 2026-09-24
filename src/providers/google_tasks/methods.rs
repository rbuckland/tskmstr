use std::collections::HashSet;

use anyhow::Result;
use log::debug;
use regex::Regex;
use reqwest::{
    header::{HeaderMap, AUTHORIZATION},
    Client,
};
use serde_json::json;

use super::model::{GoogleTask, GoogleTaskList, GoogleTasksConfig, GoogleTasksListResponse};
use crate::providers::common::{
    model::{Issue, IssueDetail, Label},
    oauth::{refresh_access_token, OAuth2RefreshConfig},
};
use crate::providers::common::{credentials::HasSecretToken, model::Comment};

fn construct_google_tasks_header(access_token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("{}{}", "Bearer ", access_token).parse().unwrap(),
    );
    headers
}

async fn get_access_token(config: &GoogleTasksConfig) -> Result<String> {
    refresh_access_token(&OAuth2RefreshConfig {
        client_id: config.oauth2.client_id.clone(),
        client_secret: config.oauth2.client_secret.clone(),
        refresh_token: config.get_token(),
        token_endpoint: config.oauth2.token_endpoint.clone(),
    })
    .await
}

fn list_url(config: &GoogleTasksConfig, tasklist: &GoogleTaskList) -> String {
    let optional_filter = tasklist
        .filter
        .as_ref()
        .map_or(String::new(), |filter| format!("&{}", filter));
    format!(
        "{}/tasks/v1/lists/{}/tasks?showCompleted=false&showDeleted=false&showHidden=false{}",
        config.endpoint, tasklist.tasklist_id, optional_filter
    )
}

fn task_url(config: &GoogleTasksConfig, tasklist: &GoogleTaskList, task_id: &str) -> String {
    format!(
        "{}/tasks/v1/lists/{}/tasks/{}",
        config.endpoint, tasklist.tasklist_id, task_id
    )
}

fn issue_url(task: &GoogleTask) -> String {
    task.web_view_link
        .clone()
        .or_else(|| task.self_link.clone())
        .unwrap_or_else(|| "https://tasks.google.com".to_string())
}

fn task_title(task: &GoogleTask) -> String {
    task.title
        .clone()
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| "(untitled task)".to_string())
}

fn tag_regex() -> Regex {
    Regex::new(r"(?i)(?:^|[\s,])#([A-Za-z0-9][A-Za-z0-9._-]*)").unwrap()
}

fn extract_tags(notes: Option<&str>) -> Vec<Label> {
    let Some(notes) = notes else {
        return Vec::new();
    };

    let mut seen = HashSet::new();
    tag_regex()
        .captures_iter(notes)
        .filter_map(|capture| capture.get(1).map(|m| m.as_str().to_string()))
        .filter(|tag| seen.insert(tag.to_ascii_lowercase()))
        .map(|name| Label { name })
        .collect()
}

fn footer_line(notes: &str) -> Option<&str> {
    notes
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .filter(|line| line.contains('#'))
}

fn render_tag_footer(tags: &[String]) -> Option<String> {
    if tags.is_empty() {
        None
    } else {
        Some(
            tags.iter()
                .map(|tag| format!("#{}", tag))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

fn normalize_tag(tag: &str) -> String {
    tag.trim().trim_start_matches('#').to_string()
}

fn upsert_tag_footer(notes: Option<&str>, tags: &HashSet<String>, add: bool) -> Option<String> {
    let normalized: HashSet<String> = tags
        .iter()
        .map(|tag| normalize_tag(tag))
        .filter(|tag| !tag.is_empty())
        .collect();

    let existing_notes = notes.unwrap_or("").trim_end_matches('\n');
    let mut body_lines: Vec<&str> = existing_notes.lines().collect();
    let existing_footer = footer_line(existing_notes);
    if existing_footer.is_some() {
        body_lines.pop();
    }

    let mut current_tags: Vec<String> = extract_tags(existing_footer)
        .into_iter()
        .map(|label| label.name)
        .collect();
    for tag in normalized {
        let has_tag = current_tags.iter().any(|existing| existing == &tag);
        if add && !has_tag {
            current_tags.push(tag);
        } else if !add {
            current_tags.retain(|existing| existing != &tag);
        }
    }
    current_tags.sort();

    let mut rebuilt = body_lines.join("\n").trim().to_string();
    if let Some(footer) = render_tag_footer(&current_tags) {
        if !rebuilt.is_empty() {
            rebuilt.push_str("\n\n");
        }
        rebuilt.push_str(&footer);
    }

    if rebuilt.trim().is_empty() {
        None
    } else {
        Some(rebuilt)
    }
}

async fn patch_task(
    tasklist: &GoogleTaskList,
    config: &GoogleTasksConfig,
    task_id: &str,
    body: serde_json::Value,
) -> Result<()> {
    let client = Client::new();
    let response = client
        .patch(task_url(config, tasklist, task_id))
        .headers(construct_google_tasks_header(
            &get_access_token(config).await?,
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Unable to update Google Task {}/{}. Status: {}",
            tasklist.id,
            task_id,
            response.status()
        );
    }

    Ok(())
}

pub async fn collect_tasks_from_google_tasks(
    google_tasks_config: &[GoogleTasksConfig],
    issue_store_id: &Option<String>,
) -> Result<Vec<Issue>> {
    let client = Client::new();
    let mut all_issues = Vec::new();

    for config in google_tasks_config {
        let access_token = get_access_token(config).await?;
        for tasklist in config.tasklists.iter().filter(|tasklist| {
            issue_store_id.is_none() || issue_store_id.as_deref().is_some_and(|p| tasklist.id == p)
        }) {
            let url = list_url(config, tasklist);
            debug!("google-tasks:get tasks {}", url);

            let response = client
                .get(&url)
                .headers(construct_google_tasks_header(&access_token))
                .send()
                .await?;

            if !response.status().is_success() {
                anyhow::bail!(
                    "Unable to fetch tasks for Google tasklist {}. Status: {}",
                    tasklist.tasklist_id,
                    response.status()
                );
            }

            let tasks: GoogleTasksListResponse = response.json().await?;
            all_issues.extend(tasks.items.into_iter().map(|task| Issue {
                id: format!("{}/{}", tasklist.id, task.id),
                color: Some(tasklist.color.clone()),
                title: task_title(&task),
                html_url: issue_url(&task),
                tags: extract_tags(task.notes.as_deref()),
            }));
        }
    }

    Ok(all_issues)
}

pub async fn add_new_task_google_tasks(
    tasklist: &GoogleTaskList,
    config: &GoogleTasksConfig,
    title: &str,
    details: &str,
    tags: &Option<Vec<String>>,
) -> Result<()> {
    let client = Client::new();
    let mut notes = details.trim().to_string();
    if let Some(tags) = tags {
        let tag_set: HashSet<String> = tags.iter().cloned().collect();
        notes = upsert_tag_footer(Some(&notes), &tag_set, true).unwrap_or_default();
    }

    let response = client
        .post(format!(
            "{}/tasks/v1/lists/{}/tasks",
            config.endpoint, tasklist.tasklist_id
        ))
        .headers(construct_google_tasks_header(
            &get_access_token(config).await?,
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&json!({
            "title": title,
            "notes": if notes.trim().is_empty() { None::<String> } else { Some(notes) },
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Unable to create Google Task in {}. Status: {}",
            tasklist.id,
            response.status()
        );
    }

    let task: GoogleTask = response.json().await?;
    println!("New task created:");
    println!("Title: {}", task_title(&task));
    println!("URL: {}", issue_url(&task));
    Ok(())
}

pub async fn close_task_google_tasks(
    config: &GoogleTasksConfig,
    tasklist: &GoogleTaskList,
    task_id: &str,
) -> Result<()> {
    patch_task(tasklist, config, task_id, json!({ "status": "completed" })).await?;
    println!(
        "Task {} closed in Google Tasks list: {}",
        task_id, tasklist.tasklist_id
    );
    Ok(())
}

pub async fn add_labels_to_google_task(
    tasklist: &GoogleTaskList,
    config: &GoogleTasksConfig,
    task_id: &str,
    tags: &HashSet<String>,
) -> Result<()> {
    let detail = fetch_google_task(config, tasklist, task_id).await?;
    patch_task(
        tasklist,
        config,
        task_id,
        json!({ "notes": upsert_tag_footer(detail.notes.as_deref(), tags, true) }),
    )
    .await
}

pub async fn remove_labels_from_google_task(
    tasklist: &GoogleTaskList,
    config: &GoogleTasksConfig,
    task_id: &str,
    tags: &HashSet<String>,
) -> Result<()> {
    let detail = fetch_google_task(config, tasklist, task_id).await?;
    patch_task(
        tasklist,
        config,
        task_id,
        json!({ "notes": upsert_tag_footer(detail.notes.as_deref(), tags, false) }),
    )
    .await
}

pub async fn add_comment_to_google_task(
    _tasklist: &GoogleTaskList,
    _config: &GoogleTasksConfig,
    _task_id: &str,
    _comment: &str,
) -> Result<()> {
    anyhow::bail!("Google Tasks does not support comments")
}

async fn fetch_google_task(
    config: &GoogleTasksConfig,
    tasklist: &GoogleTaskList,
    task_id: &str,
) -> Result<GoogleTask> {
    let client = Client::new();
    let response = client
        .get(task_url(config, tasklist, task_id))
        .headers(construct_google_tasks_header(
            &get_access_token(config).await?,
        ))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Unable to fetch task {}/{} from Google Tasks. Status: {}",
            tasklist.id,
            task_id,
            response.status()
        );
    }

    Ok(response.json().await?)
}

pub async fn view_issue_google_task(
    config: &GoogleTasksConfig,
    tasklist: &GoogleTaskList,
    task_id: &str,
) -> Result<IssueDetail> {
    let task = fetch_google_task(config, tasklist, task_id).await?;
    let body = task.notes.clone().filter(|notes| !notes.trim().is_empty());

    Ok(IssueDetail {
        issue: Issue {
            id: format!("{}/{}", tasklist.id, task.id),
            color: Some(tasklist.color.clone()),
            title: task_title(&task),
            html_url: issue_url(&task),
            tags: extract_tags(task.notes.as_deref()),
        },
        state: task.status.clone(),
        body,
        author: None,
        created_at: None,
        updated_at: task.updated.clone(),
        comments: Vec::<Comment>::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::{extract_tags, upsert_tag_footer};
    use std::collections::HashSet;

    #[test]
    fn extracts_hash_tags_from_notes() {
        let tags = extract_tags(Some("hello\n\n#home, #urgent #Work"));
        let names: Vec<String> = tags.into_iter().map(|tag| tag.name).collect();
        assert_eq!(names, vec!["home", "urgent", "Work"]);
    }

    #[test]
    fn appends_new_footer_when_missing() {
        let tags = HashSet::from(["home".to_string(), "urgent".to_string()]);
        assert_eq!(
            upsert_tag_footer(Some("buy milk"), &tags, true),
            Some("buy milk\n\n#home, #urgent".to_string())
        );
    }

    #[test]
    fn removes_selected_tags_from_footer() {
        let tags = HashSet::from(["urgent".to_string()]);
        assert_eq!(
            upsert_tag_footer(Some("buy milk\n\n#home, #urgent"), &tags, false),
            Some("buy milk\n\n#home".to_string())
        );
    }
}
