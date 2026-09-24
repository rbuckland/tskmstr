use colored::Color;
use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use std::collections::HashSet;
use std::str::FromStr;

use crate::providers::github::model::{GitHubConfig, GitHubRepository};
use crate::providers::gitlab::model::{GitLabConfig, GitLabRepository};
use crate::providers::google_tasks::model::{GoogleTaskList, GoogleTasksConfig};
use crate::providers::jira::model::{JiraConfig, JiraProject};

#[serde_inline_default]
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub debug: Option<bool>,

    pub colors: Colors,

    pub labels: LabelConfig,

    /// How `tskmstr list` (and the tray panel) orders and groups issues
    #[serde_inline_default(OutputOrdering::default())]
    pub output_ordering: OutputOrdering,

    #[serde_inline_default(Vec::<GitHubConfig>::new())]
    #[serde(rename = "github.com")]
    pub github_com: Vec<GitHubConfig>,

    #[serde_inline_default(Vec::<GitLabConfig>::new())]
    #[serde(rename = "gitlab.com")]
    pub gitlab_com: Vec<GitLabConfig>,

    #[serde_inline_default(Vec::<JiraConfig>::new())]
    pub jira: Vec<JiraConfig>,

    #[serde_inline_default(Vec::<GoogleTasksConfig>::new())]
    pub google_tasks: Vec<GoogleTasksConfig>,
}

/// Controls how the task list is grouped and ordered.
///
/// ```yaml
/// output_ordering:
///   grouped_by_tags: true       # one group per tag set (default) or one flat list
///   show_tag_heading: true      # print the "Tag: ..." heading above each tag group
/// ```
///
/// Priority-labelled issues come first, under their own heading, when there
/// are any. Within every group issues are ordered by store (config order),
/// then newest first.
#[serde_inline_default]
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct OutputOrdering {
    /// Group the non-priority issues by their tag set (default). When false,
    /// or when `show_tag_heading` is false, all non-priority issues are shown
    /// as one list.
    #[serde_inline_default(true)]
    pub grouped_by_tags: bool,

    /// Print the "Tag: a, b" heading and divider above each tag group. When
    /// false the tag groups are merged into one list, separated from the
    /// priority group by a divider line.
    #[serde_inline_default(true)]
    pub show_tag_heading: bool,
}

impl Default for OutputOrdering {
    fn default() -> Self {
        OutputOrdering {
            grouped_by_tags: true,
            show_tag_heading: true,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct LabelConfig {
    pub priority_labels: HashSet<String>,
    #[allow(dead_code)]
    pub priority_timeframe: Option<String>,
}

/// there is only one default "place" we will create tasks into
#[derive(Debug, Deserialize, Clone)]
pub enum TaskIssueProvider {
    GitHub(GitHubConfig, GitHubRepository),
    GitLab(GitLabConfig, GitLabRepository),
    Jira(JiraConfig, JiraProject),
    GoogleTasks(GoogleTasksConfig, GoogleTaskList),
}

/// Configure a task/issue source as default for some behaviour
#[derive(Debug, Deserialize, Clone)]
pub struct Defaults {
    /// One and only one Source can be set as the default for creating new tasks
    pub for_new_tasks: Option<bool>,
    /// Set this repository to show in the quick list.
    /// If this field is NOT set on any provider, then all will be displayed
    #[allow(dead_code)]
    pub for_display: Option<bool>,
}

pub trait IssueTaskRepository {
    fn defaults(&self) -> Option<Defaults>;

    fn id(&self) -> String;

    #[allow(dead_code)]
    fn color(&self) -> Color;

    fn is_default(&self) -> bool {
        match self.defaults() {
            None => false,
            Some(d) => d.for_new_tasks.unwrap_or(false),
        }
    }
}

impl AppConfig {
    pub fn find_provider_for_issue(
        &self,
        issue: &String,
    ) -> Result<Option<TaskIssueProvider>, anyhow::Error> {
        let maybe_provider: Vec<&str> = issue.split('/').collect();
        let p = String::from(*maybe_provider.first().unwrap_or_else(|
            | panic!("oops: the issue ID {} appears invalid. It was not prefixed with one of the Providers {:?}", issue, self.provider_ids())
        ));

        self.find_by(|repo: Box<&dyn IssueTaskRepository>| repo.id() == p)
    }

    pub fn find_provider_by_id(
        &self,
        provider_id: &str,
    ) -> Result<Option<TaskIssueProvider>, anyhow::Error> {
        self.find_by(|repo: Box<&dyn IssueTaskRepository>| repo.id() == *provider_id)
    }

    /// Called after configuration is loaded. It determines the unique
    /// IDs for all Task/Issue providers
    // Function to get a Vec<String> of all provider IDs
    pub fn provider_ids(&self) -> Vec<String> {
        let mut provider_ids = Vec::new();

        // Check if the GitHub configuration is present
        for g in &self.github_com {
            for repo in &g.repositories {
                provider_ids.push(repo.id.clone());
            }
        }

        // Check if the GitLab configuration is present
        for g in &self.gitlab_com {
            for repo in &g.repositories {
                provider_ids.push(repo.id.clone());
            }
        }

        // Check if the GitLab configuration is present
        for jc in &self.jira {
            for p in &jc.projects {
                provider_ids.push(p.id.clone());
            }
        }

        for gt in &self.google_tasks {
            for tasklist in &gt.tasklists {
                provider_ids.push(tasklist.id.clone());
            }
        }

        provider_ids
    }

    pub fn find_default_provider(&self) -> Result<Option<TaskIssueProvider>, anyhow::Error> {
        self.find_by(|repo: Box<&dyn IssueTaskRepository>| repo.is_default())
    }

    pub fn find_by<F: Fn(Box<&dyn IssueTaskRepository>) -> bool>(
        &self,
        f: F,
    ) -> Result<Option<TaskIssueProvider>, anyhow::Error> {
        for g in &self.github_com {
            if let Some(default_repo) = g.repositories.iter().find(|&repo| f(Box::new(repo))) {
                return Ok(Some(TaskIssueProvider::GitHub(
                    g.clone(),
                    default_repo.clone(),
                )));
            }
        }

        for g in &self.gitlab_com {
            if let Some(default_repo) = g.repositories.iter().find(|&repo| f(Box::new(repo))) {
                return Ok(Some(TaskIssueProvider::GitLab(
                    g.clone(),
                    default_repo.clone(),
                )));
            }
        }

        for jc in &self.jira {
            if let Some(found_jira_project_default) =
                jc.projects.iter().find(|&project| f(Box::new(project)))
            {
                return Ok(Some(TaskIssueProvider::Jira(
                    jc.clone(),
                    found_jira_project_default.clone(),
                )));
            }
        }

        for gt in &self.google_tasks {
            if let Some(found_tasklist) = gt.tasklists.iter().find(|&tasklist| f(Box::new(tasklist)))
            {
                return Ok(Some(TaskIssueProvider::GoogleTasks(
                    gt.clone(),
                    found_tasklist.clone(),
                )));
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Deserialize)]
pub struct Colors {
    pub issue_id: String,
    pub title: String,
    pub tags: String,
}

impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig {
            debug: None,
            github_com: Vec::new(),
            gitlab_com: Vec::new(),
            jira: Vec::new(),
            google_tasks: Vec::new(),
            labels: LabelConfig {
                priority_labels: HashSet::new(),
                priority_timeframe: None,
            },
            output_ordering: OutputOrdering::default(),
            colors: Colors {
                issue_id: "magenta".to_string(),
                title: "blue".to_string(),
                tags: "green".to_string(),
            },
        }
    }
}

/// Default configuration file location: `$HOME/.config/tskmstr/tskmstr.config.yml`
/// (`%USERPROFILE%` is used when `HOME` is not set, as is usual on Windows).
pub fn default_config_path() -> std::path::PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .unwrap_or_else(|| panic!("neither HOME nor USERPROFILE environment variable is set"));
    std::path::PathBuf::from(home).join(".config/tskmstr/tskmstr.config.yml")
}

/// Resolve the config path from an optional `--config` override.
pub fn resolve_config_path(override_path: &Option<String>) -> std::path::PathBuf {
    match override_path {
        Some(x) => std::path::PathBuf::from(x),
        None => default_config_path(),
    }
}

/// Read and parse the YAML configuration file.
pub fn load_config(path: &std::path::Path) -> Result<AppConfig, anyhow::Error> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to open file {}: {}", path.display(), e))?;
    let config: AppConfig = serde_yaml::from_str(&contents)
        .map_err(|e| anyhow::anyhow!("Failed to load file {}: {}", path.display(), e))?;
    Ok(config)
}

/// The kind of backend a configured provider talks to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    GitHub,
    GitLab,
    Jira,
    GoogleTasks,
}

impl ProviderKind {
    /// The top-level YAML key that holds providers of this kind.
    pub fn config_key(&self) -> &'static str {
        match self {
            ProviderKind::GitHub => "github.com",
            ProviderKind::GitLab => "gitlab.com",
            ProviderKind::Jira => "jira",
            ProviderKind::GoogleTasks => "google_tasks",
        }
    }

    /// The YAML key, inside a provider entry, that holds its issue stores.
    pub fn stores_key(&self) -> &'static str {
        match self {
            ProviderKind::GitHub | ProviderKind::GitLab => "repositories",
            ProviderKind::Jira => "projects",
            ProviderKind::GoogleTasks => "tasklists",
        }
    }

    /// The `provider_id` a provider of this kind gets when none is configured
    /// (mirrors the `serde_inline_default` values on the config structs).
    pub fn default_provider_id(&self) -> &'static str {
        match self {
            ProviderKind::GitHub => "github.com",
            ProviderKind::GitLab => "gitlab.com",
            ProviderKind::Jira => "jira",
            ProviderKind::GoogleTasks => "google-tasks",
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderKind::GitHub => write!(f, "github"),
            ProviderKind::GitLab => write!(f, "gitlab"),
            ProviderKind::Jira => write!(f, "jira"),
            ProviderKind::GoogleTasks => write!(f, "google"),
        }
    }
}

/// A configured provider (one keyring credential + endpoint) as shown by
/// `tskmstr issue-stores list-providers`.
#[derive(Debug, Clone)]
pub struct ProviderSummary {
    pub provider_id: String,
    pub kind: ProviderKind,
    pub endpoint: String,
    /// Short IDs of the issue stores configured under this provider
    pub store_ids: Vec<String>,
}

impl AppConfig {
    /// All configured providers, in config file order (GitHub, GitLab, Jira).
    pub fn providers(&self) -> Vec<ProviderSummary> {
        let mut out = Vec::new();

        for g in &self.github_com {
            out.push(ProviderSummary {
                provider_id: g.provider_id.clone(),
                kind: ProviderKind::GitHub,
                endpoint: g.endpoint.clone(),
                store_ids: g.repositories.iter().map(|r| r.id.clone()).collect(),
            });
        }

        for g in &self.gitlab_com {
            out.push(ProviderSummary {
                provider_id: g.provider_id.clone(),
                kind: ProviderKind::GitLab,
                endpoint: g.endpoint.clone(),
                store_ids: g.repositories.iter().map(|r| r.id.clone()).collect(),
            });
        }

        for j in &self.jira {
            out.push(ProviderSummary {
                provider_id: j.provider_id.clone(),
                kind: ProviderKind::Jira,
                endpoint: j.endpoint.clone(),
                store_ids: j.projects.iter().map(|p| p.id.clone()).collect(),
            });
        }

        for gt in &self.google_tasks {
            out.push(ProviderSummary {
                provider_id: gt.provider_id.clone(),
                kind: ProviderKind::GoogleTasks,
                endpoint: gt.endpoint.clone(),
                store_ids: gt.tasklists.iter().map(|tasklist| tasklist.id.clone()).collect(),
            });
        }

        out
    }
}

/// Add a new issue store (repository / project) under an existing provider and
/// write the updated config back to `path`.
///
/// * `provider_id` must match the `provider_id` of a configured provider
///   (see `tskmstr issue-stores list-providers`).
/// * `target` is `owner/repo` for GitHub, `group/project` (or a numeric project
///   id) for GitLab, and the project key for Jira.
/// * `color` must be a name `colored` understands (red, blue, bright green, ...).
///
/// The original file is copied to `<path>.bak` before it is rewritten. The
/// rewrite is done via `serde_yaml::Value`, so YAML comments are not preserved.
pub fn add_issue_store(
    path: &std::path::Path,
    config: &AppConfig,
    shortcode: &str,
    provider_id: &str,
    target: &str,
    color: &str,
) -> Result<(), anyhow::Error> {
    if shortcode.is_empty() || shortcode.contains('/') || shortcode.contains(char::is_whitespace) {
        anyhow::bail!(
            "Invalid shortcode '{}': it must be non-empty and contain no '/' or whitespace",
            shortcode
        );
    }

    if config.provider_ids().iter().any(|id| id == shortcode) {
        anyhow::bail!(
            "An issue store with the id '{}' already exists. Existing ids: {:?}",
            shortcode,
            config.provider_ids()
        );
    }

    if Color::from_str(color).is_err() {
        anyhow::bail!(
            "Unknown color '{}'. Use one of: black, red, green, yellow, blue, magenta, cyan, white \
             (optionally prefixed with 'bright ')",
            color
        );
    }

    let providers = config.providers();
    let provider = providers
        .iter()
        .find(|p| p.provider_id == provider_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No provider with id '{}' is configured. Known providers: {:?}\n\
                 (run `tskmstr issue-stores list-providers`)",
                provider_id,
                providers
                    .iter()
                    .map(|p| p.provider_id.as_str())
                    .collect::<Vec<_>>()
            )
        })?;

    // Build the new store entry for this provider kind
    let mut store = serde_yaml::Mapping::new();
    store.insert("id".into(), shortcode.into());
    store.insert("color".into(), color.into());
    match provider.kind {
        ProviderKind::GitHub => {
            let (owner, repo) = target.split_once('/').ok_or_else(|| {
                anyhow::anyhow!(
                    "GitHub repositories must be given as <owner>/<repo>, got '{}'",
                    target
                )
            })?;
            if owner.is_empty() || repo.is_empty() || repo.contains('/') {
                anyhow::bail!(
                    "GitHub repositories must be given as <owner>/<repo>, got '{}'",
                    target
                );
            }
            store.insert("owner".into(), owner.into());
            store.insert("repo".into(), repo.into());
        }
        ProviderKind::GitLab => {
            // GitLab wants the namespaced path URL-encoded (group%2Fproject)
            store.insert("project_id".into(), target.replace('/', "%2F").into());
        }
        ProviderKind::Jira => {
            if target.contains('/') {
                anyhow::bail!(
                    "Jira projects must be given as the project key (e.g. PROJ), got '{}'",
                    target
                );
            }
            store.insert("project_key".into(), target.into());
        }
        ProviderKind::GoogleTasks => {
            store.insert("tasklist_id".into(), target.into());
        }
    }

    // Load the raw YAML so that we keep every field we don't model
    let contents = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to open file {}: {}", path.display(), e))?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&contents)
        .map_err(|e| anyhow::anyhow!("Failed to load file {}: {}", path.display(), e))?;

    let kind = provider.kind;
    let root = doc
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("Config file {} is not a YAML mapping", path.display()))?;
    let entries = root
        .get_mut(kind.config_key())
        .and_then(|v| v.as_sequence_mut())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Config file has no '{}' list to add the provider to",
                kind.config_key()
            )
        })?;

    let entry = entries
        .iter_mut()
        .find(|e| {
            let configured = e
                .get("provider_id")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| kind.default_provider_id());
            configured == provider_id
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Could not find provider '{}' under '{}' in {}",
                provider_id,
                kind.config_key(),
                path.display()
            )
        })?;

    let entry_map = entry
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("Provider '{}' entry is not a YAML mapping", provider_id))?;
    let stores_key: serde_yaml::Value = kind.stores_key().into();
    let stores = entry_map
        .entry(stores_key)
        .or_insert_with(|| serde_yaml::Value::Sequence(Vec::new()));
    let stores = stores.as_sequence_mut().ok_or_else(|| {
        anyhow::anyhow!(
            "'{}' of provider '{}' is not a YAML list",
            kind.stores_key(),
            provider_id
        )
    })?;
    stores.push(serde_yaml::Value::Mapping(store));

    // Make sure the result still parses as our config before touching the file
    let serialized = serde_yaml::to_string(&doc)?;
    let _: AppConfig = serde_yaml::from_str(&serialized)
        .map_err(|e| anyhow::anyhow!("Updated config would be invalid, not saving: {}", e))?;

    let backup = path.with_extension(
        path.extension()
            .map(|e| format!("{}.bak", e.to_string_lossy()))
            .unwrap_or_else(|| "bak".to_string()),
    );
    std::fs::copy(path, &backup).map_err(|e| {
        anyhow::anyhow!(
            "Failed to back up {} to {}: {}",
            path.display(),
            backup.display(),
            e
        )
    })?;
    std::fs::write(path, serialized)
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", path.display(), e))?;

    println!(
        "Added issue store {} ({} {} via provider {}) to {}",
        shortcode,
        kind,
        target,
        provider_id,
        path.display()
    );
    println!("Previous config backed up to {}", backup.display());
    if contents.contains('#') {
        println!("Note: YAML comments are not preserved when the config is rewritten.");
    }
    Ok(())
}
