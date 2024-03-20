use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::YakManRole;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct CreateConfigPayload {
    pub config_name: String,
    pub project_uuid: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct DeleteConfigPayload {
    pub config_name: String,
    pub project_uuid: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub enum ProjectNotificationType {
    Slack { webhook_url: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct CreateProjectPayload {
    pub project_name: String,
    pub notification_settings: Option<ProjectNotificationSettings>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ProjectNotificationSettings {
    pub notification_type: ProjectNotificationType,
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
pub struct CreateYakManUserPayload {
    pub email: String,
    pub role: Option<YakManRole>,
}
