use reqwest;
use serde::{Deserialize};
use tokio;
use serde_yaml;
use serde_json::json;
use expanduser::expanduser;
use structopt::StructOpt;
use log::{debug, info, warn, error};
use regex::Regex;
use anyhow::Result;
use lazy_static::lazy_static;

use std::collections::HashMap;
use either::*;

use reqwest::{
    header::{HeaderMap, ACCEPT, USER_AGENT, AUTHORIZATION},
     Client
};

#[derive(Debug, Deserialize)]
struct AppConfig {

    debug: Option<bool>,

    #[serde(rename = "github.com")]
    github_com: Option<GitHubConfig>,

    #[serde(rename = "gitlab.com")]
    gitlab_com: Option<GitLabConfig>,
}

lazy_static!{
    /// Regex for tskmstr representations of a task across all task providers
    pub static ref TASK_ID_RE: Regex = Regex::new(r"^(?<task_source>gh|gl)(?<todosrc_idx>[0-9]+)/(?<issue_id>[A-Za-z0-9_-]+)$").unwrap();
}

#[derive(Debug, Deserialize, Clone)]
enum TodoSource {
    GitHub(u16, String), // unique idx, issue_id
    GitLab(u16, String), // unique idx, issue_id
}

impl std::str::FromStr for TodoSource {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        if let Some(task_id) = TASK_ID_RE.captures(s) {
            match &task_id["task_source"] {
                "gh" => {
                    let todosrc_idx = u16::from_str_radix(&task_id["todosrc_idx"],10).unwrap();
                    Ok(TodoSource::GitHub(todosrc_idx, task_id["issue_id"].to_string()))
                }
                "gl" => {
                    let todosrc_idx = u16::from_str_radix(&task_id["todosrc_idx"],10).unwrap();
                    Ok(TodoSource::GitLab(todosrc_idx, task_id["issue_id"].to_string()))
                }
                _ => Err(anyhow::anyhow!("Invalid task source")),            }
        } else {
            Err(anyhow::anyhow!("Invalid task ID format"))        }
    }
}

impl TodoSource {
    pub fn task_supplier<'a>(self, app_config: &'a AppConfig) -> Either<&'a GitHubRepository, &'a GitLabRepository> {
        match self {
            TodoSource::GitHub(todosrc_idx, _) => {
                let github_config = &app_config.github_com;
                let github_repo = github_config
                    .as_ref()
                    .and_then(|config| config.repositories.get(todosrc_idx as usize));
                Either::Left(github_repo.expect("Invalid GitHub index"))
            }
            TodoSource::GitLab(todosrc_idx, _) => {
                let gitlab_config = &app_config.gitlab_com;
                let gitlab_repo = gitlab_config
                    .as_ref()
                    .and_then(|config| config.repositories.get(todosrc_idx as usize));
                Either::Right(gitlab_repo.expect("Invalid GitLab index"))
            }
        }
    }
}

impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig {
            debug: None,
            github_com: None,
            gitlab_com: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GitHubConfig {
    token: String,
    repositories: Vec<GitHubRepository>,
}

#[derive(Debug, Deserialize)]
struct GitLabConfig {
    token: String,
    repositories: Vec<GitLabRepository>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepository {
    owner: String,
    repo: String,
    default: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct GitLabRepository {
    project_id: String,
    default: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
struct Issue {
    title: String,
    html_url: String,
    /// task/ issue id referencing the foreign system
    id: String,

    #[serde(rename = "labels")]
    tags: Vec<Label>,
}

#[derive(Debug, Deserialize, Clone)]
struct GitHubIssue {
    number: u32,
    title: String,
    html_url: String,

    // Use the new GitLabLabel type for tags
    labels: Vec<Label>,
}


#[derive(Debug, Deserialize, Clone)]
struct Label {
    name: String,
}


// New type for GitLab labels
#[derive(Debug, Deserialize, Clone)]
struct GitLabLabel(String);

#[derive(Debug, Deserialize, Clone)]
struct GitLabIssue {
    iid: u32,
    title: String,
    web_url: String,

    // Use the new GitLabLabel type for tags
    labels: Vec<GitLabLabel>,
}

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(short, long, default_value = "~/.config/tskmstr/tskmstr.config.yml")]    
    config: String,

    #[structopt(subcommand)]
    cmd: Option<Command>,
}


use std::str::FromStr;

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

pub fn construct_github_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "User".parse().unwrap());
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", token).parse().unwrap(),
    );
    headers.insert(ACCEPT, "application/vnd.github.v3+json".parse().unwrap());
    return headers;
}

pub fn construct_gitlab_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("PRIVATE-TOKEN", format!("{}",token).parse().unwrap());
    return headers;
}

async fn close_task(source: TodoSource, app_config: &AppConfig) -> Result<()> {
    let client = Client::new();

    // this is ugly (somthing about partial move from the todoSource - ChatGPT recommended this :-D - it does compile tbf)
    let (todosrc_idx, issue_id) = match &source {
        TodoSource::GitHub(idx, id) => (*idx, id.clone()),
        TodoSource::GitLab(idx, id) => (*idx, id.clone()),
    };

    match source {
        TodoSource::GitHub(_, _) => {
            let repo_config = source.task_supplier(&app_config).left().unwrap();
            let url = format!("https://api.github.com/repos/{}/{}/issues/{}", repo_config.owner, repo_config.repo, issue_id);
            debug!("github: will close {}", url);

            let response = client
                .patch(&url)
                .headers(construct_github_header(&app_config.github_com.as_ref().unwrap().token))
                .json(&serde_json::json!({
                    "state": "closed"
                }))
                .send()
                .await?;

            if response.status().is_success() {
                println!("Task {} closed in GitHub repo: {}/{}", issue_id, repo_config.owner, repo_config.repo );
            } else {
                println!("Error: Unable to close task {} in GitHub repo {}/{}. Status: {:?}", issue_id, repo_config.owner, repo_config.repo, response.status());
            }
        }
        TodoSource::GitLab(_, _) => {
            let repo_config = source.task_supplier(&app_config).right().unwrap();

            let url = format!("https://gitlab.com/api/v4/projects/{}/issues/{}", repo_config.project_id, issue_id);
            debug!("gitlab: will close {}", url);

            let response = client
                .put(&url)
                .headers(construct_gitlab_header(&app_config.gitlab_com.as_ref().unwrap().token))
                .json(&serde_json::json!({
                    "state_event": "close"
                }))
                .send()
                .await?;

            if response.status().is_success() {
                println!("Task {} closed in GitLab project: {}", issue_id, repo_config.project_id );
            } else {
                println!("Error: Unable to close {} issue in GitLab project: {}. Status: {:?}", issue_id, repo_config.project_id, response.status());
            }
        }
    }

    Ok(())
}

async fn collect_tasks_from_github(github_config: &GitHubConfig) -> Result<Vec<Issue>, anyhow::Error> {
    let client = Client::new();
    let mut all_issues = Vec::new();  // Create a vector to collect all issues

    for (idx, repo) in github_config.repositories.iter().enumerate() {
        let url = format!("https://api.github.com/repos/{}/{}/issues", repo.owner, repo.repo);

        let response = client
            .get(&url)
            .headers(construct_github_header(&github_config.token))
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;

            let github_issues: Vec<GitHubIssue> = serde_json::from_str(&body)?;
            let issues = github_issues.into_iter().map(|github_issue| {
                Issue {
                    id: format!("gh{}/{}",idx,github_issue.number),
                    title: github_issue.title,
                    html_url: github_issue.html_url,
                    tags: github_issue.labels,
                }
            });            
            all_issues.extend(issues);  // Add the collected issues to the vector

        } else {
            println!(
                "Error: Unable to fetch issues for {}/{}. Status: {:?}",
                repo.owner,
                repo.repo,
                response.status()
            );
        }
    }
    Ok(all_issues)  // Return the vector of collected issues

}


async fn collect_tasks_from_gitlab(gitlab_config: &GitLabConfig) -> Result<Vec<Issue>, anyhow::Error> {
    let client = Client::new();
    let mut all_issues = Vec::new();

    for (idx, repo) in gitlab_config.repositories.iter().enumerate() {
        let url = format!("https://gitlab.com/api/v4/projects/{}/issues", repo.project_id);

        let response = client
            .get(&url)
            .header("PRIVATE-TOKEN", gitlab_config.token.as_str())
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;
            debug!("{}", body);

            let gitlab_issues: Vec<GitLabIssue> = serde_json::from_str(&body)?;
            // Convert GitLab issues to the internal Issue representation
            let issues = gitlab_issues.into_iter().map(|gitlab_issue| {
                Issue {
                    id: format!("gl{}/{}",idx,gitlab_issue.iid),
                    title: gitlab_issue.title,
                    html_url: gitlab_issue.web_url,
                    tags: gitlab_issue.labels.into_iter().map(|label| Label { name: label.0 }).collect(),
                }
            });

            all_issues.extend(issues);

        } else {
            println!(
                "Error: Unable to fetch issues for project_id {}. Status: {:?}",
                repo.project_id,
                response.status()
            );
        }
    }

    Ok(all_issues)
}

fn display_tasks_in_table(issues: &Vec<Issue>) -> Result<(), anyhow::Error> {

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
                    println!("Tag: {}", tag);
                    println!("{:-<40}", "-"); // Divider line
            
                    for issue in tag_issues {
                        println!(" - {} {}", issue.id, issue.title);
                    }
            
                    println!(); // Empty line between tags
                }

                Ok(())
}

async fn aggregate_and_display_all_tasks(
    github_config: &Option<GitHubConfig>,
    gitlab_config: &Option<GitLabConfig>,
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

    let _ = display_tasks_in_table(&all_issues);

    Ok(())
}


async fn add_new_task(github_config: &GitHubConfig, title: &str, details: &str, tags: &Option<Vec<String>>) -> Result<(), anyhow::Error> {
    let client = Client::new();

    let default_repo = github_config
        .repositories
        .iter()
        .find(|repo| repo.default.unwrap_or(false))
        .expect("No default repository found");

    let add_url = format!(
        "https://api.github.com/repos/{}/{}/issues",
        default_repo.owner, default_repo.repo
    );

    let mut issue_details = json!({
        "title": title,
        "body": details,
    });

    if let Some(ts) = tags {
        issue_details["labels"] = serde_json::Value::Array(ts.iter().map(|label| serde_json::Value::String(label.clone())).collect());
    }    

    let response = client
        .post(&add_url)
        .headers(construct_github_header(&github_config.token))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(issue_details.to_string())
        .send().await?;

    if response.status().is_success() {
        let issue: Issue = response.json::<Issue>().await?;
        println!("New issue created:");
        println!("Title: {}", issue.title);
        println!("URL: {}", issue.html_url);
    } else {
        println!(
            "Error: Unable to create issue. Status: {:?}",
            response.status()
        );
    }
    Ok(())
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
    match args.cmd {

        Some(Command::Add { title, details, tags }) => add_new_task(&github_config.as_ref().unwrap(), &title, &details, &tags).await?,
        Some(Command::Close(close_cmd)) => {
            close_task(TodoSource::from_str(&close_cmd.id)?, &config).await?;
        }        
        None => aggregate_and_display_all_tasks(&github_config, &gitlab_config).await?,

    };


    Ok(())
}
