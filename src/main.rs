use anyhow::Result;
use expanduser::expanduser;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use serde_yaml;
use structopt::StructOpt;
use tokio;
mod config;
mod control;
mod output;
mod providers;

use config::AppConfig;
use control::*;
use providers::common::model::TodoSource;
use std::{str::FromStr, collections::HashSet};

use output::aggregate_and_display_all_tasks;

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(short, long, default_value = "~/.config/tskmstr/tskmstr.config.yml")]
    config: String,

    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(Debug, StructOpt)]
struct CloseCommand {
    #[structopt(help = "ID of the task to close")]
    id: String,
}

impl FromStr for CloseCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CloseCommand { id: s.to_string() })
    }
}

#[derive(StructOpt, Debug)]
enum TagsCommand {
    #[structopt(about = "Add more tags to the issue/task/item")]
    Add(TagOperationParameters),

    #[structopt(about = "Remove tags to the issue/task/item")]
    Remove(TagOperationParameters),
}

#[derive(StructOpt, Debug)]
struct TagOperationParameters {
    #[structopt(help = "ID of the task to change Tags on")]
    id: String,

    #[structopt(help = "Tag names")]
    tags: Vec<String>,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(about = "Add a new issue to the default repository")]
    Add {
        #[structopt()]
        title: String,

        #[structopt()]
        details: String,

        #[structopt()]
        provider_and_id: Option<String>,

        #[structopt(short, long)]
        tags: Option<Vec<String>>,
    },

    #[structopt(about = "Close a task")]
    Close(CloseCommand),

    #[structopt(about = "Make changes to the tags of issues")]
    Tags(TagsCommand),
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::from_args();
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
            provider_and_id,
            tags,
        }) => add_new_task(&provider_and_id, &config, &title, &details, &tags).await?,
        Some(Command::Close(close_cmd)) => {
            close_task(TodoSource::from_str(&close_cmd.id)?, &config).await?;
        }
        Some(Command::Tags(TagsCommand::Add(tag_additions))) => {
            let tag_set: &HashSet<String> = &tag_additions.tags.into_iter().collect();
            let task_source = TodoSource::from_str(&tag_additions.id)?;
            add_tags_to_task(&config, task_source, &tag_set).await?;
        }
        Some(Command::Tags(TagsCommand::Remove(tag_removals))) => {
            let tag_set: &HashSet<String> = &tag_removals.tags.into_iter().collect();
            let task_source = TodoSource::from_str(&tag_removals.id)?;
            remove_tags_from_task(&config, task_source, &tag_set).await?;
        }
        None => aggregate_and_display_all_tasks(&config, &colors).await?,
    };

    Ok(())
}
