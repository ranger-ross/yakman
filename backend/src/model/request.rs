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
    pub notification_settings: Option<ProjectNotificationType>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct CreateYakManUserPayload {
    pub email: String,
    pub role: Option<YakManRole>,
}
