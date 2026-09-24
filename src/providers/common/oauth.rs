use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct OAuth2RefreshConfig {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
    pub token_endpoint: String,
}

#[derive(Debug, Deserialize)]
struct OAuth2RefreshResponse {
    access_token: String,
}

pub async fn refresh_access_token(config: &OAuth2RefreshConfig) -> Result<String> {
    let response = Client::new()
        .post(&config.token_endpoint)
        .form(&[
            ("client_id", config.client_id.as_str()),
            ("client_secret", config.client_secret.as_str()),
            ("refresh_token", config.refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "OAuth2 token refresh failed against {}. Status: {}",
            config.token_endpoint,
            response.status()
        );
    }

    Ok(response.json::<OAuth2RefreshResponse>().await?.access_token)
}
