use async_trait::async_trait;
use serde::Serialize;
use serde_json::{json, Value};

use super::{YakManNotificationAdapter, YakManNotificationType};

#[derive(Debug, Serialize)]
struct SlackNotificationPayload {
    text: String,
}

pub struct SlackNotificationAdapter {
    pub http_client: reqwest::Client,
    pub webhook_url: String,
}

#[async_trait]
impl YakManNotificationAdapter for SlackNotificationAdapter {
    async fn send_notification(&self, event: YakManNotificationType) -> anyhow::Result<()> {
        let payload = self.create_slack_payload(&event);

        log::info!("{:#?}", payload);

        self.http_client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        return Ok(());
    }
}

impl SlackNotificationAdapter {
    fn create_slack_payload(&self, event: &YakManNotificationType) -> Value {
        match event {
            YakManNotificationType::RevisionReviewSubmitted => json!({
                "text": "Revision submitted"
            }),
        }
    }
}
