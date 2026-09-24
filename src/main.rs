use anyhow::Result;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use clap::{Parser, Subcommand};
use tskmstr::providers::jira::methods::list_jira_transition_ids;

use std::{collections::HashSet, path::PathBuf, str::FromStr};
use tskmstr::config::AppConfig;
use tskmstr::control::*;

use tskmstr::output::{
    aggregate_and_display_all_tasks, display_issue_detail, list_issue_stores, list_providers,
};

#[derive(Debug, Parser)]
#[command(name = "t")]
#[command(about = "tskmstr: A Task & Issue Management Aggregation CLI", long_about = None)]
struct Cli {
    #[arg(short, long)]
    debug: bool,

    /// Config file default is "~/.config/tskmstr/tskmstr.config.yml" (all platforms)
    /// Override with --config <path>
    #[arg(short, long)]
    config: Option<String>,

    #[command(subcommand)]
    // optional because, default execution with no args will list all tasks/issues
    cmd: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Add a new issue to the default repository
    Add {
        /// The title of your issue/task
        title: String,

        /// Details of the issue
        details: String,

        /// Tags/Labels to apply to the issue/task
        tags: Option<Vec<String>>,

        /// Limit the activity to one issue/task repository
        #[arg(short, long)]
        issue_store_id: Option<String>,
    },

    /// Close a task
    Close(CloseCommand),

    /// Add a comment/note to a issue/task
    Comment(CommentCommand),

    /// Add and remove tags/labels from issue/task
    #[command(subcommand)]
    Tags(TagsCommand),

    /// List, add and inspect issue/task stores (repositories, projects)
    IssueStores {
        #[command(subcommand)]
        // optional: `issue-stores` on its own lists the configured stores
        cmd: Option<IssueStoresCommand>,
    },

    /// Initialise a new tskmstr configuration file
    Init {
        /// Overwrite an existing config file if present
        #[arg(short, long)]
        force: bool,
    },

    /// Show Jira Transitions allowed for a given ID
    JiraTransitions { id: String },

    /// Show the full detail of one or more issues/tasks (description, comments)
    View {
        /// Issue ids to show, e.g. P/78 J/ABC-123
        #[arg(required = true)]
        ids: Vec<String>,
    },

    /// default action, list all issues
    List {
        /// Limit the activity to one issue/task provider
        #[arg(short, long)]
        issue_store_id: Option<String>,

        /// Show all details
        #[arg(short, long)]
        all: bool,
    },
}

#[derive(Debug, clap::Args)]
struct CloseCommand {
    /// ID of the issue/task to close
    id: String,
}

#[derive(Debug, clap::Args)]
struct CommentCommand {
    /// ID of the issue/task to add a comment to
    id: String,
    /// New Comment to add to the issue/task
    comment: String,
}

impl FromStr for CloseCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CloseCommand { id: s.to_string() })
    }
}

#[derive(Subcommand, Debug)]
enum IssueStoresCommand {
    /// List the configured issue/task stores (default)
    List,

    /// List the configured providers (credential + endpoint) that stores can be added to
    ListProviders,

    /// Add a new issue/task store to an existing provider in the config file
    ///
    /// Example: tskmstr issue-stores add SPG github/rbuckland_sqce sqc-internal/pretty_goat blue
    Add {
        /// Short, unique id for the store (used as the issue id prefix, e.g. SPG/12)
        shortcode: String,

        /// The provider_id of a configured provider (see `issue-stores list-providers`)
        provider: String,

        /// GitHub: <owner>/<repo>; GitLab: <group>/<project> or a numeric id; Jira: the project key
        target: String,

        /// Color used when displaying the store id (red, blue, "bright green", ...)
        #[arg(default_value = "white")]
        color: String,
    },
}

#[derive(Parser, Debug)]
enum TagsCommand {
    /// Add more tags to the issue/task/item
    Add(TagOperationParameters),

    /// Remove tags to the issue/task/item
    Remove(TagOperationParameters),
}

#[derive(clap::Args, Debug)]
struct TagOperationParameters {
    /// ID of the task (must be prefixed with the provider id e.g. P-888, or J-ID-999)
    id: String,

    // Tag/Label names
    tags: Vec<String>,
}

async fn do_work(
    args: &Cli,
    config_path: &std::path::Path,
    config: &AppConfig,
) -> Result<(), anyhow::Error> {
    // Initialize your logger
    if args.debug || config.debug.is_some() {
        // Set up the logger with the desired log level
        simple_logger::init_with_level(log::Level::Debug).expect("Failed to initialize logger");
    } else {
        // Initialize the logger with a default log level
        simple_logger::init_with_level(log::Level::Info).expect("Failed to initialize logger");
    }

    let colors = &config.colors;
    match &args.cmd {
        Some(Command::Add {
            title,
            details,
            tags,
            issue_store_id,
        }) => add_new_task(issue_store_id, config, title, details, tags).await?,
        Some(Command::Close(close_cmd)) => {
            close_task(config, close_cmd.id.clone()).await?;
        }
        Some(Command::Comment(comment_cmd)) => {
            comment_task(config, comment_cmd.id.clone(), comment_cmd.comment.clone()).await?;
        }
        Some(Command::Tags(TagsCommand::Add(tag_additions))) => {
            let tag_set: &HashSet<String> = &tag_additions.tags.clone().into_iter().collect();
            add_tags_to_task(config, tag_additions.id.clone(), tag_set).await?;
        }
        Some(Command::Tags(TagsCommand::Remove(tag_removals))) => {
            let tag_set: &HashSet<String> = &tag_removals.tags.clone().into_iter().collect();
            remove_tags_from_task(config, tag_removals.id.clone(), tag_set).await?;
        }
        Some(Command::IssueStores { cmd: None })
        | Some(Command::IssueStores {
            cmd: Some(IssueStoresCommand::List),
        }) => {
            list_issue_stores(config).await?;
        }
        Some(Command::IssueStores {
            cmd: Some(IssueStoresCommand::ListProviders),
        }) => {
            list_providers(config);
        }
        Some(Command::IssueStores {
            cmd:
                Some(IssueStoresCommand::Add {
                    shortcode,
                    provider,
                    target,
                    color,
                }),
        }) => {
            tskmstr::config::add_issue_store(
                config_path,
                config,
                shortcode,
                provider,
                target,
                color,
            )?;
        }
        Some(Command::Init { .. }) => {
            // handled in main before config loading
            unreachable!()
        }
        Some(Command::JiraTransitions { id }) => {
            list_jira_transition_ids(&config.jira[0], id).await?;
        }
        Some(Command::View { ids }) => {
            for (i, id) in ids.iter().enumerate() {
                if i > 0 {
                    println!("{:=<60}", "=");
                    println!();
                }
                let detail = view_task(config, id).await?;
                display_issue_detail(&detail, colors);
            }
        }
        Some(Command::List {
            issue_store_id,
            all,
        }) => aggregate_and_display_all_tasks(issue_store_id, config, colors, all).await?,
        None => aggregate_and_display_all_tasks(&None, config, colors, &false).await?,
    };

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();

    let config_path = tskmstr::config::resolve_config_path(&args.config);

    // Handle init before attempting to load config
    if let Some(Command::Init { force }) = &args.cmd {
        return do_init(&config_path, *force);
    }

    let config = tskmstr::config::load_config(&config_path)?;
    do_work(&args, &config_path, &config).await
}

fn do_init(config_path: &PathBuf, force: bool) -> Result<(), anyhow::Error> {
    if config_path.exists() && !force {
        anyhow::bail!(
            "Config file already exists at {}. Use --force to overwrite.",
            config_path.display()
        );
    }

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let template = r#"# tskmstr configuration
# Documentation: https://github.com/rbuckland/tskmstr

colors:
  tags: green
  title: blue
  issue_id: red

labels:
  priority_labels:
    - urgent
    - todo

# How `tskmstr list` lays out the issues (all optional, defaults shown).
# Priority-labelled issues are always listed first under their own heading.
output_ordering:
  grouped_by_tags: true       # false = one flat list after the priority group
  ordered_by_provider: false  # true = keep issues from the same store together
  show_tag_heading: true      # false = no "Tag: ..." heading above each group

# ---------------------------------------------------------------------------
# GitHub configuration  (remove this section if not used)
# ---------------------------------------------------------------------------
# Store your Personal Access Token in the OS keyring (service + username must match below):
#   macOS:  security add-generic-password -U -s github.com -a <username> -w
#   Linux:  secret-tool store --label='tskmstr github' service github.com username <username>
#   Any:    keyring set github.com <username>
# ---------------------------------------------------------------------------
github.com:
  - provider_id: github/myusername
    credential:
      service: github.com
      username: myusername
    repositories:
      - id: G
        color: blue
        owner: my-org
        repo: my-repo
        defaults:
          for_new_tasks: true
        # filter: labels=my-label

# ---------------------------------------------------------------------------
# GitLab configuration  (remove this section if not used)
# ---------------------------------------------------------------------------
# Store your Personal Access Token in the OS keyring (service + username must match below):
#   macOS:  security add-generic-password -U -s gitlab.com -a <username> -w
#   Linux:  secret-tool store --label='tskmstr gitlab' service gitlab.com username <username>
#   Any:    keyring set gitlab.com <username>
# ---------------------------------------------------------------------------
gitlab.com:
  - provider_id: gitlab/myusername
    credential:
      service: gitlab.com
      username: myusername
    repositories:
      - id: L
        color: green
        project_id: myorg%2Fmy-project
        defaults:
          for_new_tasks: false

# ---------------------------------------------------------------------------
# Jira configuration  (remove this section if not used)
# ---------------------------------------------------------------------------
# Store your API token in the OS keyring (username must be your Jira login email):
#   macOS:  security add-generic-password -U -s yourinstance.atlassian.net -a user@example.com -w
#   Linux:  secret-tool store --label='tskmstr jira' service yourinstance.atlassian.net username user@example.com
#   Any:    keyring set yourinstance.atlassian.net user@example.com
# ---------------------------------------------------------------------------
jira:
  - provider_id: My Jira
    endpoint: https://yourinstance.atlassian.net
    credential:
      service: yourinstance.atlassian.net
      username: user@example.com
    projects:
      - id: J
        color: yellow
        project_key: PROJ
        default_issue_type: Task
        close_transition_id: 31
        defaults:
          for_new_tasks: false
        # filter: assignee = currentUser()
"#;

    std::fs::write(config_path, template)?;
    println!("Created config file: {}", config_path.display());
    println!("Edit it to add your repositories, then store credentials in the OS keyring.");
    Ok(())
}
