use oauth2::PkceCodeChallenge;
use oauth2::PkceCodeVerifier;
pub use serde::Deserialize;
pub use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct Config {
    pub name: String, // Unique key
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct LabelType {
    pub name: String, // Unique key
    pub description: String,
    pub priority: i32,
    pub options: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct Label {
    pub label_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ConfigInstance {
    pub config_name: String,
    pub instance: String,   // Unique key
    pub labels: Vec<Label>, // These should match the labels in the current revision
    pub current_revision: String,
    pub pending_revision: Option<String>,
    pub revisions: Vec<String>,
    pub changelog: Vec<ConfigInstanceChange>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ConfigInstanceChange {
    pub timestamp_ms: i64,
    pub previous_revision: Option<String>,
    pub new_revision: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ConfigInstanceRevision {
    pub revision: String, // Unique key
    pub data_key: String, // Key to fetch data
    pub labels: Vec<Label>,
    pub timestamp_ms: i64,
    pub approved: bool,
    pub content_type: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct YakManSettings {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthInitPayload {
    pub challenge: PkceCodeChallenge,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthExchangePayload {
    pub state: String,
    pub code: String,
    pub verifier: PkceCodeVerifier,
}
