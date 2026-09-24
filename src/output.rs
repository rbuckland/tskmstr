use crate::config::{Colors, OutputOrdering};
use crate::providers::github::methods::collect_tasks_from_github;

use crate::providers::gitlab::methods::collect_tasks_from_gitlab;

use crate::providers::jira::methods::collect_tasks_from_jira;
use crate::{
    config::AppConfig,
    providers::common::model::{Issue, IssueDetail},
};
use colored::{Color, Colorize};
use std::str::FromStr;

use std::collections::{HashMap, HashSet};

/// A named group of issues, as shown by `tskmstr list`.
///
/// The first group returned by [`group_tasks`] is always the priority group
/// (issues carrying one of the configured priority labels), followed by one
/// group per distinct set of remaining tags, sorted alphabetically.
#[derive(Debug, Clone)]
pub struct TaskGroup {
    /// Human readable heading, e.g. `Priority: urgent, todo` or `Tag: home`
    pub heading: String,
    /// True for the priority group
    pub is_priority: bool,
    /// Whether the heading (and divider) should be rendered; driven by
    /// `output_ordering.show_tag_heading` for tag groups
    pub show_heading: bool,
    pub issues: Vec<Issue>,
}

/// Everything [`group_tasks`] needs from the config to lay out the list.
#[derive(Debug, Clone)]
pub struct DisplayOptions {
    pub priority_labels: HashSet<String>,
    pub ordering: OutputOrdering,
    /// Issue store ids in config-file order; used by `ordered_by_provider`
    pub store_order: Vec<String>,
}

impl DisplayOptions {
    pub fn from_config(config: &AppConfig) -> Self {
        DisplayOptions {
            priority_labels: config.labels.priority_labels.clone(),
            ordering: config.output_ordering.clone(),
            store_order: config.provider_ids(),
        }
    }

    /// Position of the issue's store in the config, for stable sorting.
    /// Unknown stores sort after all known ones.
    fn store_rank(&self, issue: &Issue) -> usize {
        let store = issue.id.split('/').next().unwrap_or("");
        self.store_order
            .iter()
            .position(|s| s == store)
            .unwrap_or(self.store_order.len())
    }
}

// Function to group tasks by labels, excluding priority labels
fn group_tasks_by_labels(
    issues: &[Issue],
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
    issues: &[Issue],
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

/// Group issues exactly the way `tskmstr list` displays them:
/// priority-labelled issues first, then the remaining issues either grouped
/// by their (non-priority) tag set and sorted by group name, or as one flat
/// list, according to `output_ordering`.
pub fn group_tasks(issues: &[Issue], opts: &DisplayOptions) -> Vec<TaskGroup> {
    let priority_labels = &opts.priority_labels;
    let priority_tasks = group_tasks_by_priority_labels(issues, priority_labels);

    let mut priority_label_names: Vec<String> = priority_labels.iter().cloned().collect();
    priority_label_names.sort();

    let mut result = vec![TaskGroup {
        heading: format!("Priority: {}", priority_label_names.join(", ")),
        is_priority: true,
        show_heading: true,
        issues: priority_tasks,
    }];

    if opts.ordering.grouped_by_tags {
        let mut grouped_tasks = group_tasks_by_labels(issues, priority_labels);
        grouped_tasks.remove(""); // Remove the empty key
        let mut groups: Vec<_> = grouped_tasks.into_iter().collect();
        groups.sort_by(|(a, _), (b, _)| a.cmp(b));

        result.extend(groups.into_iter().map(|(name, issues)| TaskGroup {
            heading: format!("Tag: {}", name),
            is_priority: false,
            show_heading: opts.ordering.show_tag_heading,
            issues,
        }));
    } else {
        // One flat list of everything that is not a priority task
        let others: Vec<Issue> = issues
            .iter()
            .filter(|issue| {
                issue
                    .tags
                    .iter()
                    .all(|tag| !priority_labels.contains(&tag.name))
            })
            .cloned()
            .collect();
        result.push(TaskGroup {
            heading: "Tasks".to_string(),
            is_priority: false,
            show_heading: opts.ordering.show_tag_heading,
            issues: others,
        });
    }

    if opts.ordering.ordered_by_provider {
        for group in &mut result {
            // stable: issues from the same store keep their provider order
            group.issues.sort_by_key(|issue| opts.store_rank(issue));
        }
    }

    result
}

/// The colour to paint an issue id with: the originating store's `color:` when
/// it is set and valid, otherwise the global `colors.issue_id`.
pub fn issue_id_color(issue: &Issue, colors: &Colors) -> Color {
    issue
        .color
        .as_deref()
        .and_then(|c| Color::from_str(c).ok())
        .unwrap_or_else(|| Color::from_str(&colors.issue_id).unwrap())
}

fn print_issue_line(issue: &Issue, colors: &Colors, all: &bool) {
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
        true => format!(" - {}", issue.html_url),
    };

    println!(
        " - {} {} {}{}",
        issue.id.color(issue_id_color(issue, colors)),
        issue.title.color(Color::from_str(&colors.title).unwrap()),
        tags.color(Color::from_str(&colors.tags).unwrap()),
        details
    );
}

pub fn display_tasks_in_table(
    issues: &[Issue],
    colors: &Colors,
    opts: &DisplayOptions,
    all: &bool,
) -> Result<(), anyhow::Error> {
    let tag_color = Color::from_str(&colors.tags).unwrap();

    for group in group_tasks(issues, opts) {
        if group.show_heading {
            // Heading is "<Kind>: <names>"; colour only the names part, as before.
            match group.heading.split_once(": ") {
                Some((kind, names)) => println!("{}: {}", kind, names.color(tag_color)),
                None => println!("{}", group.heading),
            }
            println!("{:-<40}", "-"); // Divider line
        }
        for issue in &group.issues {
            print_issue_line(issue, colors, all);
        }
        if group.show_heading || !group.issues.is_empty() {
            println!();
        }
    }

    Ok(())
}

/// Collect open issues from every configured provider (optionally limited to
/// a single issue store) into one list.
pub async fn collect_all_tasks(
    provider_id: &Option<String>,
    config: &AppConfig,
) -> Result<Vec<Issue>, anyhow::Error> {
    let mut all_issues = Vec::new();

    let github_tasks = collect_tasks_from_github(&config.github_com, provider_id).await?;
    all_issues.extend(github_tasks);

    let gitlab_tasks = collect_tasks_from_gitlab(&config.gitlab_com, provider_id).await?;
    all_issues.extend(gitlab_tasks);

    let jira_tasks = collect_tasks_from_jira(&config.jira, provider_id).await?;
    all_issues.extend(jira_tasks);

    Ok(all_issues)
}

pub async fn aggregate_and_display_all_tasks(
    provider_id: &Option<String>,
    config: &AppConfig,
    colors: &Colors,
    all: &bool,
) -> Result<(), anyhow::Error> {
    let all_issues = collect_all_tasks(provider_id, config).await?;
    display_tasks_in_table(
        &all_issues,
        colors,
        &DisplayOptions::from_config(config),
        all,
    )
}

/// Render one issue in full (`tskmstr view <id>`): header, description and
/// comments. Colours follow the same config as `list`.
pub fn display_issue_detail(detail: &IssueDetail, colors: &Colors) {
    let issue = &detail.issue;
    let tag_color = Color::from_str(&colors.tags).unwrap();
    let title_color = Color::from_str(&colors.title).unwrap();

    println!(
        "{} {}",
        issue.id.color(issue_id_color(issue, colors)).bold(),
        issue.title.color(title_color).bold()
    );
    println!("{}", issue.html_url.dimmed());
    println!();

    if let Some(state) = &detail.state {
        println!("{:<9}{}", "State:", state);
    }
    let labels = issue
        .tags
        .iter()
        .map(|t| t.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    if !labels.is_empty() {
        println!("{:<9}{}", "Labels:", labels.color(tag_color));
    }
    if let Some(author) = &detail.author {
        println!("{:<9}{}", "Author:", author);
    }
    if let Some(created) = &detail.created_at {
        println!("{:<9}{}", "Created:", created);
    }
    if let Some(updated) = &detail.updated_at {
        println!("{:<9}{}", "Updated:", updated);
    }

    println!();
    match &detail.body {
        Some(body) => println!("{}", body.trim_end()),
        None => println!("{}", "(no description)".dimmed()),
    }

    println!();
    println!("Comments ({})", detail.comments.len());
    println!("{:-<40}", "-");
    for comment in &detail.comments {
        let who = comment.author.as_deref().unwrap_or("unknown");
        let when = comment.created_at.as_deref().unwrap_or("");
        println!("{}", format!("[{} @ {}]", who, when).color(tag_color));
        println!("{}", comment.body.trim_end());
        println!();
    }
}

/// Print the configured providers (credential + endpoint), one per line, with
/// the issue store ids that live under each. The first column is the value to
/// pass to `tskmstr issue-stores add <shortcode> <provider> ...`.
pub fn list_providers(config: &AppConfig) {
    let providers = config.providers();
    let id_width = providers
        .iter()
        .map(|p| p.provider_id.chars().count())
        .max()
        .unwrap_or(0);

    for p in providers {
        let stores = if p.store_ids.is_empty() {
            "(no issue stores)".to_string()
        } else {
            p.store_ids.join(", ")
        };
        println!(
            "{:<width$}  {:<6}  {}  [{}]",
            p.provider_id,
            p.kind.to_string(),
            p.endpoint,
            stores,
            width = id_width
        );
    }
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
