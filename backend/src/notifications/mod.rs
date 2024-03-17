pub mod slack;

use std::sync::Arc;

use async_trait::async_trait;

use crate::model::NotificationSetting;

use self::slack::SlackNotificationAdapter;

#[async_trait]
pub trait YakManNotificationAdapter {
    async fn send_notification(&self, event: YakManNotificationType) -> anyhow::Result<()>;
}

pub enum YakManNotificationType {
    RevisionReviewSubmitted {
        project_name: String,
        config_name: String,
        instance: String,
        revision: String,
    },
    RevisionReviewApproved {
        project_name: String,
        config_name: String,
        instance: String,
        revision: String,
    },
    RevisionReviewApplied {
        project_name: String,
        config_name: String,
        instance: String,
        revision: String,
    },
    RevisionReviewRejected {
        project_name: String,
        config_name: String,
        instance: String,
        revision: String,
    },
}

impl From<NotificationSetting> for Arc<dyn YakManNotificationAdapter + Send + Sync> {
    fn from(value: NotificationSetting) -> Self {
        match value {
            NotificationSetting::Slack { webhook_url } => Arc::new(SlackNotificationAdapter {
                http_client: reqwest::Client::new(),
                webhook_url: webhook_url,
            }),
        }
    }
}
