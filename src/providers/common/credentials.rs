
use keyring::Entry;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
/// application, service and username of an entry in the OS keyring
pub struct CredentialKeyringEntry {
    pub service: String,
    pub username: String,
}
pub trait HasSecretToken {
    fn task_provider_id(&self) -> String;

    fn token(&self) -> Option<String>;

    fn credential(&self) -> Option<CredentialKeyringEntry>;

    fn get_token(&self) -> String {
        let token_str = match (self.token(), self.credential()) {
            (None, Some(cke)) => Entry::new(&cke.service, &cke.username)
                .expect(
                    format!(
                        "failed to get the keyring for {}/{}",
                        &cke.service, &cke.username
                    )
                    .as_str(),
                )
                .get_password()
                .expect(
                    format!(
                        "failed to get the API token for {}/{}",
                        &cke.service, &cke.username
                    )
                    .as_str(),
                ),
            (Some(token), None) => token,
            _ => panic!(
                "Please provide a token or credentials in config for: {}",
                self.task_provider_id()
            ),
        };

        token_str
    }
}
