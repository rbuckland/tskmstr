use reqwest;
use serde::{Deserialize, Serialize};
use tokio;
use serde_yaml;
use expanduser::expanduser;

use clap::Parser;

use reqwest::{
    header::{HeaderMap, ACCEPT, USER_AGENT, AUTHORIZATION},
    Url, Client
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "~/.config/tskmstr/tskmstr.config.yml")]    
    config: String,
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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::parse();

    // Read the repository configuration from YAML
    let config_file = std::fs::read_to_string(expanduser(args.config)?)?;
    let config: RepositoryConfig = serde_yaml::from_str(&config_file)?;

    let github_config = &config.github_com;
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
