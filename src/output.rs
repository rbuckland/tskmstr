use crate::config::Colors;
use crate::providers::github::methods::collect_tasks_from_github;

use crate::providers::gitlab::methods::collect_tasks_from_gitlab;

use crate::providers::jira::methods::collect_tasks_from_jira;
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

            acc.entry(group_key).or_default().push(issue.clone());
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
    all: &bool,
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

        let details = match all { 
            false => "".to_string(),
            true => format!(" - {}",issue.html_url)
        };

        println!(
            " - {} {} {}{}",
            issue.id.color(Color::from_str(&colors.issue_id).unwrap()),
            issue.title.color(Color::from_str(&colors.title).unwrap()),
            tags.color(Color::from_str(&colors.tags).unwrap()),
            details
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
                let details = match all { 
                    false => "".to_string(),
                    true => format!(" - {}",issue.html_url)
                };

                println!(
                    " - {} {} {}{}",
                    issue.id.color(Color::from_str(&colors.issue_id).unwrap()),
                    issue.title.color(Color::from_str(&colors.title).unwrap()),
                    tags.color(Color::from_str(&colors.tags).unwrap()),
                    details
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
    all: &bool,
) -> Result<(), anyhow::Error> {
    let mut all_issues = Vec::new();

    let github_tasks = collect_tasks_from_github(&config.github_com, provider_id).await?;
    all_issues.extend(github_tasks);

    let gitlab_tasks = collect_tasks_from_gitlab(&config.gitlab_com, provider_id).await?;
    all_issues.extend(gitlab_tasks);

    let jira_tasks = collect_tasks_from_jira(&config.jira, provider_id).await?;
    all_issues.extend(jira_tasks);

    display_tasks_in_table(&all_issues, colors, &config.labels.priority_labels, all)

}

pub async fn list_issue_stores(config: &AppConfig) -> Result<(), anyhow::Error> {
    
    for g in &config.github_com {
        for x in &g.repositories {
            println!("{} - {}/{}/{}", x.id, g.endpoint, x.owner, x.repo);
        }
    }

    for g in &config.gitlab_com {
        for x in &g.repositories {
            println!("{} - {}/{}", x.id, g.endpoint, x.project_id);
        }
    }

    for g in &config.jira {
        for x in &g.projects {
            println!("{} - {}/{}", x.id, g.endpoint, x.id);
        }
    }

    Ok(())
}
