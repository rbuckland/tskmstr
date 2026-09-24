use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

fn percent_encode(input: &str) -> String {
    input
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            b' ' => vec!['+'],
            _ => format!("%{:02X}", byte).chars().collect(),
        })
        .collect()
}

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
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(format!(
            "client_id={}&client_secret={}&refresh_token={}&grant_type=refresh_token",
            percent_encode(&config.client_id),
            percent_encode(&config.client_secret),
            percent_encode(&config.refresh_token)
        ))
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
