use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{NotificationSetting, NotificationSettingEvents, YakManProjectRole, YakManRole};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct CreateConfigPayload {
    pub config_name: String,
    pub project_id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct DeleteConfigPayload {
    pub config_id: String,
    pub project_id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub enum ProjectNotificationType {
    Slack { webhook_url: String },
    Discord { webhook_url: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct CreateProjectPayload {
    pub project_name: String,
    pub notification_settings: Option<ProjectNotificationSettings>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct UpdateProjectPayload {
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

impl Into<crate::model::ProjectNotificationSettings> for ProjectNotificationSettings {
    fn into(self) -> crate::model::ProjectNotificationSettings {
        let events = NotificationSettingEvents {
            is_instance_updated_enabled: self.is_instance_updated_enabled,
            is_instance_created_enabled: self.is_instance_created_enabled,
            is_revision_submitted_enabled: self.is_revision_submitted_enabled,
            is_revision_approved_enabled: self.is_revision_approved_enabled,
            is_revision_reject_enabled: self.is_revision_reject_enabled,
        };

        let settings = match self.notification_type {
            ProjectNotificationType::Slack { webhook_url } => NotificationSetting::Slack {
                webhook_url: webhook_url,
            },
            ProjectNotificationType::Discord { webhook_url } => NotificationSetting::Discord {
                webhook_url: webhook_url,
            },
        };
        crate::model::ProjectNotificationSettings { settings, events }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct CreateYakManUserPayload {
    pub email: String,
    pub role: Option<YakManRole>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct CreateTeamPayload {
    pub name: String,
    pub global_roles: Vec<YakManRole>,
    pub roles: Vec<YakManProjectRole>,
}
