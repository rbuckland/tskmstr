use reqwest;
use serde::{Deserialize};
use tokio;
use serde_yaml;
use serde_json::json;
use expanduser::expanduser;
use structopt::StructOpt;

use reqwest::{
    header::{HeaderMap, ACCEPT, USER_AGENT, AUTHORIZATION},
     Client
};

#[derive(Debug, Deserialize)]
struct RepositoryConfig {
    #[serde(rename = "github.com")]
    github_com: GitHubConfig,
}

#[derive(Debug, Deserialize)]
struct GitHubConfig {
    token: String,
    repositories: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
struct Repository {
    owner: String,
    repo: String,
    default: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Issue {
    title: String,
    html_url: String,
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

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(about = "Add a new issue to the default repository")]
    Add {
        #[structopt()]
        title: String,
        
        #[structopt()]
        details: String,
    }
}

pub fn construct_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "User".parse().unwrap());
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", token).parse().unwrap(),
    );
    headers.insert(ACCEPT, "application/vnd.github.v3+json".parse().unwrap());
    return headers;
}

async fn list_tasks(github_config: &GitHubConfig) -> Result<(), Box<dyn std::error::Error>> {

    let client = Client::new();

    for repo in &github_config.repositories {
        let url = format!("https://api.github.com/repos/{}/{}/issues", repo.owner, repo.repo);

        let response = client.get(&url).headers(construct_header(&github_config.token)).send().await?;

        // Check if the response status is successful
        if response.status().is_success() {
            let body = response.text().await?;
            let issues: Vec<Issue> = serde_json::from_str(&body)?;

            println!("Issues for {}/{}:", repo.owner, repo.repo);
            for issue in issues {
                println!("{} - {}", issue.title, issue.html_url);
            }
        } else {
            println!(
                "Error: Unable to fetch issues for {}/{}. Status: {:?}",
                repo.owner,
                repo.repo,
                response.status()
            );
        }
    }
    Ok(())
}

async fn add_new_task(github_config: &GitHubConfig, title: &str, details: &str) -> Result<(), Box<dyn std::error::Error>> {

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

    let issue_details = json!({
        "title": title,
        "body": details,
    });

    let response = client
        .post(&add_url)
        .headers(construct_header(&github_config.token))
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::from_args();

    // Read the repository configuration from YAML
    let config_file = std::fs::read_to_string(expanduser(&args.config)?)?;
    let config: RepositoryConfig = serde_yaml::from_str(&config_file)?;

    let github_config = &config.github_com;
    match args.cmd {

        Some(Command::Add { title, details }) => add_new_task(&github_config, &title, &details).await?,
        None => list_tasks(&github_config).await?,

    };


    Ok(())
}
