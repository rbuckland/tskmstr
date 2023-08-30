use anyhow::Result;
use expanduser::expanduser;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use serde_yaml;
use structopt::StructOpt;
use tokio;
mod config;
mod providers;
mod output;
mod control;

use std::str::FromStr;
use providers::common::model::TodoSource;
use config::AppConfig;
use control::close_task;
use providers::github::methods::add_new_task;

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
enum Command {
    #[structopt(about = "Add a new issue to the default repository")]
    Add {
        #[structopt()]
        title: String,

        #[structopt()]
        details: String,

        #[structopt(short, long)]
        tags: Option<Vec<String>>,
    },

    #[structopt(about = "Close a task")]
    Close(CloseCommand),
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

    let github_config = &config.github_com;
    let gitlab_config = &config.gitlab_com;
    let colors = &config.colors;
    match args.cmd {
        Some(Command::Add {
            title,
            details,
            tags,
        }) => add_new_task(&github_config.as_ref().unwrap(), &title, &details, &tags).await?,
        Some(Command::Close(close_cmd)) => {
            close_task(TodoSource::from_str(&close_cmd.id)?, &config).await?;
        }
        None => aggregate_and_display_all_tasks(&github_config, &gitlab_config, &colors).await?,
    };

    Ok(())
}
