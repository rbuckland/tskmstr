use crate::config::{Colors, ProviderIface};
use crate::providers::github::methods::collect_tasks_from_github;
use crate::providers::github::model::GitHubConfig;
use crate::providers::gitlab::methods::collect_tasks_from_gitlab;
use crate::providers::gitlab::model::GitLabConfig;
use crate::{config::AppConfig, providers::common::model::Issue};
use colored::{Color, Colorize};
use std::str::FromStr;

use std::collections::{HashMap, HashSet};

// Function to group tasks by labels, excluding priority labels
fn group_tasks_by_labels(
    issues: &Vec<Issue>,
    priority_labels: &HashSet<String>,
) -> HashMap<String, Vec<Issue>> {
    issues
        .iter()
        .filter(|issue| {
            issue
                .tags
                .iter()
                .all(|tag| !priority_labels.contains(&tag.name))
        })
        .fold(HashMap::new(), |mut acc, issue| {
            let labels_except_priority: Vec<String> = issue
                .tags
                .iter()
                .filter(|tag| !priority_labels.contains(&tag.name))
                .map(|tag| tag.name.clone())
                .collect();

            let group_key = if labels_except_priority.is_empty() {
                "<no labels>".to_string()
            } else {
                labels_except_priority.join(", ")
            };

            acc.entry(group_key)
                .or_insert_with(Vec::new)
                .push(issue.clone());
            acc
        })
}

fn group_tasks_by_priority_labels(
    issues: &Vec<Issue>,
    priority_labels: &HashSet<String>,
) -> Vec<Issue> {
    issues
        .iter()
        .filter(|issue| {
            issue
                .tags
                .iter()
                .any(|tag| priority_labels.contains(&tag.name))
        })
        .cloned()
        .collect()
}

pub fn display_tasks_in_table(
    issues: &Vec<Issue>,
    colors: &Colors,
    priority_labels: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    let mut grouped_tasks = group_tasks_by_labels(issues, priority_labels);
    let priority_tasks = group_tasks_by_priority_labels(issues, priority_labels);

    // Sort the groups by priority, moving the priority group to the front
    grouped_tasks.remove(""); // Remove the empty key
    let mut groups: Vec<_> = grouped_tasks.iter().collect();
    groups.sort_by(|(a, _), (b, _)| {
        if a == &"priority" {
            std::cmp::Ordering::Less
        } else if b == &"priority" {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });

    // Display priority tasks
    let priority_labels_str = &priority_labels
        .iter()
        .cloned()
        .collect::<Vec<String>>()
        .join(", ");
    println!(
        "Priority: {}",
        priority_labels_str.color(Color::from_str(&colors.tags).unwrap())
    );
    println!("{:-<40}", "-"); // Divider line
    for issue in &priority_tasks {
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

    println!();

    // Display grouped tasks
    for (group, group_issues) in groups {
        if !group.is_empty() {
            println!(
                "Tag: {}",
                group.color(Color::from_str(&colors.tags).unwrap())
            );
            println!("{:-<40}", "-"); // Divider line
            for issue in group_issues {
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
            println!();
        }
    }

    Ok(())
}

pub async fn aggregate_and_display_all_tasks(
    provider_id: &Option<String>,
    config: &AppConfig,
    colors: &Colors,
) -> Result<(), anyhow::Error> {
    let mut all_issues = Vec::new();

    if let Some(github_config) = &config.github_com {
        let github_tasks = collect_tasks_from_github(github_config, &provider_id).await?;
        all_issues.extend(github_tasks);
    }

    if let Some(gitlab_config) = &config.gitlab_com {
        let gitlab_tasks = collect_tasks_from_gitlab(gitlab_config, &provider_id).await?;
        all_issues.extend(gitlab_tasks);
    }

    let _ = display_tasks_in_table(&all_issues, &colors, &config.labels.priority_labels);

    Ok(())
}

pub async fn list_providers(
    config: &AppConfig,
) -> Result<(), anyhow::Error> {
    let mut all_providers: Vec<Box<dyn ProviderIface>> = Vec::new();

    if let Some(github_config) = &config.github_com {
        for x in &github_config.repositories {
            println!("{} - github.com/{}/{}",x.id,x.owner,x.repo);
        }
    }

    if let Some(gitlab_config) = &config.gitlab_com {
        for x in &gitlab_config.repositories {
            println!("{} - gitlab.com/{}",x.id,x.project_id);
        }
    }

    Ok(())
}