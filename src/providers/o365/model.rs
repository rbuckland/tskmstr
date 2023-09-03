
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct O365Task {
    // Define the fields of an O365 To-Do task
    // For example:
    // pub id: String,
    // pub title: String,
    // pub due_date: Option<String>,
    // ...
}

#[derive(Debug, Deserialize, Clone)]
pub struct O365Config {
    pub todo_lists: Vec<O365TodoConfiguration>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct O365TodoConfiguration {
    pub name: String,
    pub application_client_id: String,
    pub object_id: String,
    pub directory_tenant_id: String,
    pub client_secret: String,
    pub default: Option<bool>,
}
