use async_trait::async_trait;
use serde_json::{json, Value};

use crate::settings;

use super::{YakManNotificationAdapter, YakManNotificationType};

pub struct DiscordNotificationAdapter {
    pub http_client: reqwest::Client,
    pub webhook_url: String,
}

#[async_trait]
impl YakManNotificationAdapter for DiscordNotificationAdapter {
    async fn send_notification(&self, event: YakManNotificationType) -> anyhow::Result<()> {
        self.http_client
            .post(&self.webhook_url)
            .json(&event.to_discord_payload())
            .send()
            .await?
            .error_for_status()?;

        return Ok(());
    }
}

impl YakManNotificationType {
    fn to_discord_payload(&self) -> Value {
        match self {
            YakManNotificationType::RevisionReviewSubmitted {
                project_name,
                config_name,
                instance,
                revision,
            } => {
                let review_request_message = if let Some(host) = settings::yakman_application_host()
                {
                    // TODO: FIX links on discord
                    format!(
                        "Revision: {revision} [Link]({host}/apply-changes/{config_name}/{instance})"
                    )
                } else {
                    format!("Revision: {revision}")
                };

                json!({
                    "embeds": [
                        {
                            "title": format!(":loudspeaker: New config change request submitted for `{project_name}`"),
                            "fields": [
                                {
                                    "name": "Config",
                                    "value": config_name,
                                    "inline": true
                                },
                                {
                                    "name": "Instance",
                                    "value": format!("`{instance}`"),
                                    "inline": true
                                }
                            ],
                            "footer": {
                                "text": review_request_message
                            }
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
                    "embeds": [
                        {
                            "title": format!(":white_check_mark: Config change request approved for `{project_name}`"),
                            "fields": [
                                {
                                    "name": "Config",
                                    "value": config_name,
                                    "inline": true
                                },
                                {
                                    "name": "Instance",
                                    "value": format!("`{instance}`"),
                                    "inline": true
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
                    "embeds": [
                        {
                            "title": format!(":rocket: Config change applied for `{project_name}`"),
                            "fields": [
                                {
                                    "name": "Config",
                                    "value": config_name,
                                    "inline": true
                                },
                                {
                                    "name": "Instance",
                                    "value": format!("`{instance}`"),
                                    "inline": true
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
                    "embeds": [
                        {
                            "title": format!(":boom: Config change rejected for `{project_name}`"),
                            "fields": [
                                {
                                    "name": "Config",
                                    "value": config_name,
                                    "inline": true
                                },
                                {
                                    "name": "Instance",
                                    "value": format!("`{instance}`"),
                                    "inline": true
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
                    "embeds": [
                        {
                            "title": format!(":loudspeaker: Config instance created for `{project_name}`"),
                            "fields": [
                                {
                                    "name": "Config",
                                    "value": config_name,
                                    "inline": true
                                },
                                {
                                    "name": "Instance",
                                    "value": format!("`{instance}`"),
                                    "inline": true
                                }
                            ]
                        }
                    ]
                })
            }
        }
    }
}
