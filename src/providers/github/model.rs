use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubLabel {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubIssue {
    pub number: u32,
    pub title: String,
    pub html_url: String,

    // Use the new GitLabLabel type for tags
    pub labels: Vec<GitHubLabel>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubConfig {
    pub token: String,
    pub repositories: Vec<GitHubRepository>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubRepository {
    pub owner: String,
    pub repo: String,
    pub default: Option<bool>,
}
