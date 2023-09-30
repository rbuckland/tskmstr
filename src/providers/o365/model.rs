use serde::Deserialize;

use crate::config::{Defaults, ProviderIface};
use colored::Color;
use std::str::FromStr;

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
    /// a unique character across the entire repository config
    /// which will be used for display and CMD line choices
    /// If an ID is not set, an auto generated one will be created
    pub id: String,

    /// In output, Where color is appropriate, together with the ID, this will be used
    pub color: String,

    /// Which O365 Entry are we using ?
    pub list_id: String,

    /// Defauls configuration
    pub defaults: Option<Defaults>,
}

impl ProviderIface for O365TodoList {
    fn defaults(&self) -> Option<Defaults> {
        self.defaults.clone()
    }

    fn color(&self) -> Color {
        Color::from_str(&self.color).unwrap()
    }

    fn id(&self) -> String {
        self.id.clone()
    }
}
