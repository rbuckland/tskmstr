use serde::Deserialize;

// New type for GitLab labels
#[derive(Debug, Deserialize, Clone)]
pub struct GitLabLabel(pub String);

#[derive(Debug, Deserialize, Clone)]
pub struct GitLabIssue {
    pub iid: u32,
    pub title: String,
    pub web_url: String,

    // Use the new GitLabLabel type for tags
    pub labels: Vec<GitLabLabel>,
}

#[derive(Debug, Deserialize)]
pub struct GitLabConfig {
    pub token: String,
    pub repositories: Vec<GitLabRepository>,
}

#[derive(Debug, Deserialize)]
pub struct GitLabRepository {
    pub project_id: String,
    pub default: Option<bool>,
}
