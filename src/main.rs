#![feature(fn_traits)]
#![feature(unboxed_closures)]


use anyhow::Result;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use clap::{Parser, Subcommand};
use providers::jira::methods::list_jira_transition_ids;



mod config;
mod control;
mod output;
mod providers;

use config::AppConfig;
use control::*;
use std::{collections::HashSet, path::PathBuf, str::FromStr};

use output::{aggregate_and_display_all_tasks, list_issue_stores};

use directories::ProjectDirs;

#[derive(Debug, Parser)]
#[command(name = "t")]
#[command(about = "tskmstr: A Task & Issue Management Aggregation CLI", long_about = None)]
struct Cli {
    #[arg(short, long)]
    debug: bool,

    /// Config file default is "~/.config/tskmstr/tskmstr.config.yml"
    /// For Windows %LOCALAPPDATA%/tskmstr/tskmstr.config.yml
    /// For OSX ~/Library/Preferences/tskmstr/tskmstr.config.yml
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

    /// Add and remove tags/labels from issue/task
    #[command(subcommand)]
    Tags(TagsCommand),

    /// List issue/task stores
    IssueStores,

    /// Show Jira Transitions allowed for a given ID
    JiraTransitions { id: String },

    /// default action, list all issues
    List {

        /// Limit the activity to one issue/task provider
        #[arg(short, long)]
        issue_store_id: Option<String>,

        /// Show all details
        #[arg(short, long)]
        all: bool,
    }
}

#[derive(Debug, clap::Args)]
struct CloseCommand {
    #[arg(help = "ID of the task to close")]
    id: String,

}

impl FromStr for CloseCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CloseCommand { id: s.to_string()})
    }
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

async fn do_work(args: &Cli, config: &AppConfig) -> Result<(), anyhow::Error> {
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
        Some(Command::Tags(TagsCommand::Add(tag_additions))) => {
            let tag_set: &HashSet<String> = &tag_additions.tags.clone().into_iter().collect();
            add_tags_to_task(config, tag_additions.id.clone(), tag_set).await?;
        }
        Some(Command::Tags(TagsCommand::Remove(tag_removals))) => {
            let tag_set: &HashSet<String> = &tag_removals.tags.clone().into_iter().collect();
            remove_tags_from_task(config, tag_removals.id.clone(), tag_set).await?;
        }
        Some(Command::IssueStores) => {
            list_issue_stores(config).await?;
        }
        Some(Command::JiraTransitions { id }) => {
            list_jira_transition_ids(&config.jira[0], id).await?;
        }
        Some(Command::List{ issue_store_id , all }) => aggregate_and_display_all_tasks(issue_store_id, config, colors, &all).await?,
        None => aggregate_and_display_all_tasks(&None, config, colors, &false).await?,
    };

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();
    // Read the repository configuration from YAML

    let config_file = match &args.config {
        Some(x) => PathBuf::from(x).as_os_str().to_owned(),
        None => {
            let proj_dirs = ProjectDirs::from("org", "inosion", "tskmstr")
                .unwrap_or_else(|| panic!("No Config directory found"));
            let mut config_dir = proj_dirs.config_dir().as_os_str().to_owned();
            config_dir.push("/tskmstr.config.yml");
            config_dir
        }
    };
    let filename = config_file.clone().into_string().unwrap();

    let contents = std::fs::read_to_string(&config_file)
        .expect(format!("Failed to open file {}", filename).as_str());
    let config: AppConfig = serde_yaml::from_str(&contents)
        .expect(format!("Failed to load file {}", filename).as_str());
    do_work(&args, &config).await
}
