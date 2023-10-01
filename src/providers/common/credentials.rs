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

    fn credential(&self) -> Option<CredentialKeyringEntry>;

    fn get_username(&self) -> String {
        self.credential().unwrap().username.clone()
    }

    fn get_token(&self) -> String {
        

        match self.credential() {
            Some(cke) => Entry::new(&cke.service, &cke.username)
                .unwrap_or_else(|_| panic!("failed to get the keyring for {}/{}",
                        &cke.service, &cke.username))
                .get_password()
                .unwrap_or_else(|_| panic!("failed to get the API token for {}/{}",
                        &cke.service, &cke.username)),
            _ => panic!(
                "Please provide a credentials in config for: {}",
                self.task_provider_id()
            ),
        }
    }
}
