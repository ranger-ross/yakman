pub mod oauth;
pub mod request;

pub use serde::Deserialize;
pub use serde::Serialize;
use std::fmt;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct YakManProject {
    pub uuid: String, // Unique key
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct Config { // TODO: Rename to YakManConfig
    pub name: String, // Unique key
    pub project_uuid: String,
    pub description: String,
    #[serde(default)]
    pub hidden: bool,
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
    pub applied_by_uuid: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub enum RevisionReviewState {
  Pending,
  Approved,
  Rejected
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ConfigInstanceRevision {
    pub revision: String, // Unique key
    pub data_key: String, // Key to fetch data
    pub labels: Vec<Label>,
    pub timestamp_ms: i64,
    pub review_state: RevisionReviewState,
    pub reviewed_by_uuid: Option<String>,
    pub review_timestamp_ms: Option<i64>,
    pub content_type: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct YakManSettings {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone)]
pub enum YakManRole {
    Admin,
    Approver,
    Operator,
    Viewer,
}

impl fmt::Display for YakManRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YakManRole::Admin => write!(f, "Admin"),
            YakManRole::Approver => write!(f, "Approver"),
            YakManRole::Operator => write!(f, "Operator"),
            YakManRole::Viewer => write!(f, "Viewer"),
        }
    }
}

impl TryFrom<Option<String>> for YakManRole {
    type Error = &'static str;

    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        if let Some(value) = value {
            return YakManRole::try_from(value);
        }
        Err("Invalid role")
    }
}

impl TryFrom<String> for YakManRole {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        return match value.as_str() {
            "Admin" => Ok(YakManRole::Admin),
            "Approver" => Ok(YakManRole::Approver),
            "Operator" => Ok(YakManRole::Operator),
            "Viewer" => Ok(YakManRole::Viewer),
            _ => Err("Invalid role"),
        };
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct YakManUser {
    pub email: String,
    pub uuid: String,
    pub role: Option<YakManRole>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone)]
pub struct YakManUserProjectRole {
    pub project_uuid: String,
    pub role: YakManRole,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct YakManUserDetails {
    pub global_roles: Vec<YakManRole>,
    pub roles: Vec<YakManUserProjectRole>,
}
