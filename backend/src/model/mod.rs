pub mod request;
pub mod response;

pub use serde::Deserialize;
pub use serde::Serialize;
use std::fmt;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct YakManProject {
    pub id: String, // Unique key
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct YakManProjectDetails {
    pub id: String,
    pub name: String,
    pub notification_settings: Option<ProjectNotificationSettings>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ProjectNotificationSettings {
    pub settings: NotificationSetting,
    pub events: NotificationSettingEvents,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub enum NotificationSetting {
    Slack { webhook_url: String },
    Discord { webhook_url: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct NotificationSettingEvents {
    #[serde(default)]
    pub is_instance_updated_enabled: bool,
    #[serde(default)]
    pub is_instance_created_enabled: bool,
    #[serde(default)]
    pub is_revision_submitted_enabled: bool,
    #[serde(default)]
    pub is_revision_approved_enabled: bool,
    #[serde(default)]
    pub is_revision_reject_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct YakManConfig {
    pub id: String, // Unique key
    pub name: String,
    pub project_id: String,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct LabelType {
    pub id: String, // Unique key
    pub name: String,
    pub description: String,
    pub options: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct YakManLabel {
    pub label_id: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ConfigDetails {
    pub config_id: String,
    pub config_name: String,
    pub project_id: String,
    pub instances: Vec<ConfigInstance>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ConfigInstance {
    pub config_id: String,
    pub instance: String,         // Unique key
    pub labels: Vec<YakManLabel>, // These should match the labels in the current revision
    pub current_revision: String,
    pub pending_revision: Option<String>,
    pub revisions: Vec<String>,
    pub changelog: Vec<ConfigInstanceEvent>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ConfigInstanceEvent {
    #[serde(flatten)]
    pub event: ConfigInstanceEventData,
    pub timestamp_ms: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub enum ConfigInstanceEventData {
    Created {
        new_revision: String,
        created_by_user_id: String,
    },
    Updated {
        previous_revision: String,
        new_revision: String,
        applied_by_user_id: String,
    },
    NewRevisionSubmitted {
        previous_revision: String,
        new_revision: String,
        submitted_by_user_id: String,
    },
    NewRevisionApproved {
        new_revision: String,
        approver_by_user_id: String,
    },
    NewRevisionRejected {
        new_revision: String,
        rejected_by_user_id: String,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub enum RevisionReviewState {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ConfigInstanceRevision {
    pub revision: String, // Unique key
    pub data_key: String, // Key to fetch data
    pub labels: Vec<YakManLabel>,
    pub timestamp_ms: i64,
    pub review_state: RevisionReviewState,
    pub reviewed_by_user_id: Option<String>,
    pub review_timestamp_ms: Option<i64>,
    pub submitted_by_user_id: String,
    pub submit_timestamp_ms: i64,
    pub content_type: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct YakManApiKey {
    pub id: String,
    pub hash: String,
    pub project_id: String,
    pub role: YakManRole,
    pub created_at: i64,
    pub created_by_user_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Clone, Hash)]
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
    pub id: String,
    pub role: Option<YakManRole>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Clone, Hash)]
pub struct YakManProjectRole {
    pub project_id: String,
    pub role: YakManRole,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct YakManUserDetails {
    pub user_id: String,
    pub profile_picture: Option<String>,
    pub global_roles: Vec<YakManRole>,
    pub roles: Vec<YakManProjectRole>,
    pub team_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YakManPassword {
    pub hash: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YakManPasswordResetLink {
    pub email_hash: String,
    pub expiration_timestamp_ms: i64,
}

/// Public response when creating a password reset link
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct YakManPublicPasswordResetLink {
    pub id: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct YakManTeam {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct YakManTeamDetails {
    pub id: String,
    pub name: String,
    pub global_roles: Vec<YakManRole>,
    pub roles: Vec<YakManProjectRole>,
    pub member_user_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YakManSnapshotLock {
    pub lock: Option<YakManSnapshotLockInner>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YakManSnapshotLockInner {
    pub id: String,
    pub timestamp_ms: i64,
}

impl YakManSnapshotLock {
    pub fn new(id: String, timestamp_ms: i64) -> Self {
        Self {
            lock: Some(YakManSnapshotLockInner { id, timestamp_ms }),
        }
    }
    /// Creates an unlocked lock file
    pub fn unlocked() -> Self {
        Self { lock: None }
    }
}
