pub mod auth;
pub mod methods;
pub mod model;


/// o365 has an underscore, as we create unique ID's like o365_1/issue_id
/// the 5 and a number would clash
pub const SHORT_CODE_O365: &str = "o365_";