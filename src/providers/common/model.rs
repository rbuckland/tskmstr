use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Issue {
    /// The title of the issue
    pub title: String,

    /// Originating URL
    pub html_url: String,

    /// task/ issue id referencing the foreign system
    pub id: String,

    /// List of labels, or tags
    #[serde(rename = "labels")]
    pub tags: Vec<Label>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Label {
    pub name: String,
}
