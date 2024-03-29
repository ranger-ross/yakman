use async_trait::async_trait;
use serde_json::{json, Value};

use crate::settings;

use super::{YakManNotificationAdapter, YakManNotificationType};

pub struct SlackNotificationAdapter {
    pub http_client: reqwest::Client,
    pub webhook_url: String,
}

#[async_trait]
impl YakManNotificationAdapter for SlackNotificationAdapter {
    async fn send_notification(&self, event: YakManNotificationType) -> anyhow::Result<()> {
        self.http_client
            .post(&self.webhook_url)
            .json(&event.to_slack_payload())
            .send()
            .await?
            .error_for_status()?;

        return Ok(());
    }
}

impl YakManNotificationType {
    fn to_slack_payload(&self) -> Value {
        match self {
            YakManNotificationType::RevisionReviewSubmitted {
                project_name,
                config_name,
                instance,
                revision,
            } => {
                let review_request_message = if let Some(host) = settings::yakman_application_host()
                {
                    format!(
                        "Revision: {revision} <{host}/apply-changes/{config_name}/{instance}|Link>"
                    )
                } else {
                    format!("Revision: {revision}")
                };

                json!({
                    "blocks": [
                        {
                            "type": "section",
                            "text": {
                                "type": "mrkdwn",
                                "text": format!(":loudspeaker: *New config change request submitted for `{project_name}`*")
                            }
                        },
                        {
                            "type": "section",
                            "fields": [
                                {
                                    "type": "mrkdwn",
                                    "text": format!("*Config:* {config_name}")
                                },
                                {
                                    "type": "mrkdwn",
                                    "text": format!("*Instance:* `{instance}`")
                                }
                            ]
                        },
                        {
                            "type": "context",
                            "elements": [
                                {
                                    "type": "mrkdwn",
                                    "text": review_request_message
                                }
                            ]
                        }
                    ]
                })
            }
            YakManNotificationType::RevisionReviewApproved {
                project_name,
                config_name,
                instance,
                revision: _,
            } => {
                json!({
                    "blocks": [
                        {
                            "type": "section",
                            "text": {
                                "type": "mrkdwn",
                                "text": format!(":white_check_mark: *Config change request approved for `{project_name}`*")
                            }
                        },
                        {
                            "type": "section",
                            "fields": [
                                {
                                    "type": "mrkdwn",
                                    "text": format!("*Config:* {config_name}")
                                },
                                {
                                    "type": "mrkdwn",
                                    "text": format!("*Instance:* `{instance}`")
                                }
                            ]
                        }
                    ]
                })
            }
            YakManNotificationType::RevisionReviewApplied {
                project_name,
                config_name,
                instance,
                revision: _,
            } => {
                json!({
                    "blocks": [
                        {
                            "type": "section",
                            "text": {
                                "type": "mrkdwn",
                                "text": format!(":rocket: *Config change applied for `{project_name}`*")
                            }
                        },
                        {
                            "type": "section",
                            "fields": [
                                {
                                    "type": "mrkdwn",
                                    "text": format!("*Config:* {config_name}")
                                },
                                {
                                    "type": "mrkdwn",
                                    "text": format!("*Instance:* `{instance}`")
                                }
                            ]
                        }
                    ]
                })
            }
            YakManNotificationType::RevisionReviewRejected {
                project_name,
                config_name,
                instance,
                revision: _,
            } => {
                json!({
                    "blocks": [
                        {
                            "type": "section",
                            "text": {
                                "type": "mrkdwn",
                                "text": format!(":boom: *Config change rejected for `{project_name}`*")
                            }
                        },
                        {
                            "type": "section",
                            "fields": [
                                {
                                    "type": "mrkdwn",
                                    "text": format!("*Config:* {config_name}")
                                },
                                {
                                    "type": "mrkdwn",
                                    "text": format!("*Instance:* `{instance}`")
                                }
                            ]
                        }
                    ]
                })
            }
            YakManNotificationType::InstanceCreated {
                project_name,
                config_name,
                instance,
            } => {
                json!({
                    "blocks": [
                        {
                            "type": "section",
                            "text": {
                                "type": "mrkdwn",
                                "text": format!(":loudspeaker: *Config instance created for `{project_name}`*")
                            }
                        },
                        {
                            "type": "section",
                            "fields": [
                                {
                                    "type": "mrkdwn",
                                    "text": format!("*Config:* {config_name}")
                                },
                                {
                                    "type": "mrkdwn",
                                    "text": format!("*Instance:* `{instance}`")
                                }
                            ]
                        }
                    ]
                })
            }
        }
    }
}
