
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
    pub client_id: String,
    pub client_secret: String,
    pub todo_lists: Vec<O365TodoList>,

}
#[derive(Debug, Deserialize, Clone)]
pub struct O365TodoList {
    pub list_id: String,
    pub default: Option<bool>,
}