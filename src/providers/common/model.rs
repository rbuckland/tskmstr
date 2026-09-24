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

    /// Display colour of the issue store this came from (the store's `color:`
    /// in config). `None` means "use the global `colors.issue_id`".
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Label {
    pub name: String,
}

/// A single comment / note on an issue, as shown by `tskmstr view`.
#[derive(Debug, Clone)]
pub struct Comment {
    pub author: Option<String>,
    /// Creation timestamp as the provider reports it (ISO 8601 string)
    pub created_at: Option<String>,
    pub body: String,
}

/// Full detail of one issue, as shown by `tskmstr view <id>`.
/// All providers map their native issue representation into this.
#[derive(Debug, Clone)]
pub struct IssueDetail {
    /// The summary fields, identical to what `tskmstr list` shows
    pub issue: Issue,
    /// Provider state / status name, e.g. `open`, `opened`, `In Progress`
    pub state: Option<String>,
    /// The issue description / body, if any
    pub body: Option<String>,
    pub author: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    /// Comments in chronological order
    pub comments: Vec<Comment>,
}
