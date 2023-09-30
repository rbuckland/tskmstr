#![feature(fn_traits)]
#![feature(unboxed_closures)]

use anyhow::Result;
use expanduser::expanduser;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use serde_yaml;
use clap::{Args, Parser, Subcommand};

use tokio;
mod config;
mod control;
mod output;
mod providers;

use config::AppConfig;
use control::*;
use providers::common::{model::TaskIssueProviderConfig};
use std::{str::FromStr, collections::HashSet};

use output::{aggregate_and_display_all_tasks, list_providers};


#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "t")]
#[command(about = "tskmstr: A Task & Issue Management CLI", long_about = None)]
struct Cli {
    #[arg(short, long)]
    debug: bool,

    #[arg(short, long, default_value = "~/.config/tskmstr/tskmstr.config.yml")]
    config: String,

    /// Limit the activity to one task/issue provider
    #[arg(short, long)]
    provider_id: Option<String>,

    #[command(subcommand)]
    // optional because, default execution with no args will list all tasks/issues
    cmd: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Add a new issue to the default repository
    Add {
        /// The title of your task/issue
        title: String,

        /// Details of the issue
        details: String,

        /// Tags/Labels to apply to the issue/task
        tags: Option<Vec<String>>,
    },

    /// Close a task
    Close(CloseCommand),

    /// Add and remove tags/labels of issues/tasks
    #[command(subcommand)]
    Tags(TagsCommand),

    /// List Providers
    Providers,

}

#[derive(Debug, clap::Args)]
struct CloseCommand {
    #[arg(help = "ID of the task to close")]
    id: String,
}

impl FromStr for CloseCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CloseCommand { id: s.to_string() })
    }
}

#[derive(Parser, Debug)]
enum TagsCommand {

    /// Add more tags to the issue/task/item
    Add(TagOperationParameters),

    /// Remove tags to the issue/task/item
    Remove(TagOperationParameters)
}


#[derive(clap::Args, Debug)]
struct TagOperationParameters {

    /// ID of the task (must be prefixed with the provider id e.g. P-888, or J-ID-999)
    id: String,

    // Tag/Label names
    tags: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();
    // Read the repository configuration from YAML
    let config_file = std::fs::read_to_string(expanduser(&args.config)?)?;
    let config: AppConfig = serde_yaml::from_str(&config_file)?;

    // Initialize your logger
    if args.debug || config.debug.is_some() {
        // Set up the logger with the desired log level
        simple_logger::init_with_level(log::Level::Debug).expect("Failed to initialize logger");
    } else {
        // Initialize the logger with a default log level
        simple_logger::init_with_level(log::Level::Info).expect("Failed to initialize logger");
    }

    let colors = &config.colors;
    match args.cmd {
        Some(Command::Add {
            title,
            details,
            tags,
        }) => add_new_task(&args.provider_id, &config, &title, &details, &tags).await?,
        Some(Command::Close(close_cmd)) => {
            close_task(TaskIssueProviderConfig::from_str(&close_cmd.id)?, &config).await?;
        }
        Some(Command::Tags(TagsCommand::Add(tag_additions))) => {
            let tag_set: &HashSet<String> = &tag_additions.tags.into_iter().collect();
            let task_source = TaskIssueProviderConfig::from_str(&tag_additions.id)?;
            add_tags_to_task(&config, task_source, &tag_set).await?;
        }
        Some(Command::Tags(TagsCommand::Remove(tag_removals))) => {
            let tag_set: &HashSet<String> = &tag_removals.tags.into_iter().collect();
            let task_source = TaskIssueProviderConfig::from_str(&tag_removals.id)?;
            remove_tags_from_task(&config, task_source, &tag_set).await?;
        }
        Some(Command::Providers) => {
            list_providers(&config).await?;
        }
        None => aggregate_and_display_all_tasks(&args.provider_id, &config, &colors).await?,
    };

    Ok(())
}
