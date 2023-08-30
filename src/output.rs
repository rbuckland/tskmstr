use std::collections::HashMap;
use crate::providers::common::model::Issue;
use crate::providers::gitlab::model::GitLabConfig;
use crate::providers::github::model::GitHubConfig;
use crate::providers::github::methods::collect_tasks_from_github;
use crate::providers::gitlab::methods::collect_tasks_from_gitlab;
use crate::config::Colors;
use colored::{Colorize, Color};
use std::str::FromStr;

pub fn display_tasks_in_table(issues: &Vec<Issue>, colors: &Colors) -> Result<(), anyhow::Error> {
    // Group issues by tags
    let mut issues_by_tags: HashMap<String, Vec<Issue>> = HashMap::new();
    for issue in issues {
        for tag in &issue.tags {
            let tag_name = &tag.name;
            issues_by_tags
                .entry(tag_name.to_string())
                .or_insert_with(Vec::new)
                .push(issue.clone());
        }
    }

    for (tag, tag_issues) in &issues_by_tags {
        println!("Tag: {}", tag.color(Color::from_str(&colors.tags).unwrap()));
        println!("{:-<40}", "-"); // Divider line

        for issue in tag_issues {
            let tags = format!(
                "({})",
                issue
                    .tags
                    .iter()
                    .map(|t| t.name.clone())
                    .collect::<Vec<String>>()
                    .join(", ")
            );
            println!(
                " - {} {} {}",
                issue.id.color(Color::from_str(&colors.issue_id).unwrap()),
                issue.title.color(Color::from_str(&colors.title).unwrap()),
                tags.color(Color::from_str(&colors.tags).unwrap())
            );
        }

        println!(); // Empty line between tags
    }

    Ok(())
}


pub async fn aggregate_and_display_all_tasks(
    github_config: &Option<GitHubConfig>,
    gitlab_config: &Option<GitLabConfig>,
    colors: &Colors,
) -> Result<(), anyhow::Error> {
    let mut all_issues = Vec::new();

    if let Some(github_config) = github_config {
        let github_tasks = collect_tasks_from_github(github_config).await?;
        all_issues.extend(github_tasks);
    }

    if let Some(gitlab_config) = gitlab_config {
        let gitlab_tasks = collect_tasks_from_gitlab(gitlab_config).await?;
        all_issues.extend(gitlab_tasks);
    }

    let _ = display_tasks_in_table(&all_issues, &colors);

    Ok(())
}

