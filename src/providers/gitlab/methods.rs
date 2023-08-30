use crate::providers::common::model::Issue;
use crate::providers::gitlab::model::GitLabConfig;
use crate::providers::gitlab::model::GitLabIssue;
use crate::providers::gitlab::SHORT_CODE_GITLAB;
use anyhow::Result;

use crate::providers::common::model::Label;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use reqwest::{header::HeaderMap, Client};

pub fn construct_gitlab_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("PRIVATE-TOKEN", format!("{}", token).parse().unwrap());
    return headers;
}

pub async fn collect_tasks_from_gitlab(
    gitlab_config: &GitLabConfig,
) -> Result<Vec<Issue>, anyhow::Error> {
    let client = Client::new();
    let mut all_issues = Vec::new();

    for (idx, repo) in gitlab_config.repositories.iter().enumerate() {
        let url = format!(
            "https://gitlab.com/api/v4/projects/{}/issues",
            repo.project_id
        );

        let response = client
            .get(&url)
            .header("PRIVATE-TOKEN", gitlab_config.token.as_str())
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;
            debug!("{}", body);

            let gitlab_issues: Vec<GitLabIssue> = serde_json::from_str(&body)?;
            // Convert GitLab issues to the internal Issue representation
            let issues = gitlab_issues.into_iter().map(|gitlab_issue| Issue {
                id: format!("{}{}/{}", SHORT_CODE_GITLAB, idx, gitlab_issue.iid),
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
