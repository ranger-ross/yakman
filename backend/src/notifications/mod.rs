pub mod slack;

use async_trait::async_trait;

#[async_trait]
pub trait YakManNotificationAdapter {
    async fn send_notification(&self, event: YakManNotificationType) -> anyhow::Result<()>;
}

pub enum YakManNotificationType {
    RevisionReviewSubmitted,
}
