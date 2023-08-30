use serde::Deserialize;

use crate::providers::gitlab::model::GitLabConfig;
use crate::providers::github::model::GitHubConfig;


#[derive(Debug, Deserialize)]
pub struct AppConfig {

    pub debug: Option<bool>,

    pub colors: Colors,

    #[serde(rename = "github.com")]
    pub github_com: Option<GitHubConfig>,

    #[serde(rename = "gitlab.com")]
    pub gitlab_com: Option<GitLabConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Colors {
    pub issue_id: String,
    pub title: String,
    pub tags: String,
}



impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig {
            debug: None,
            github_com: None,
            gitlab_com: None,
            colors: Colors {
                issue_id: "magenta".to_string(),
                title: "blue".to_string(),
                tags: "green".to_string(),
            }, 
       }
    }
}
