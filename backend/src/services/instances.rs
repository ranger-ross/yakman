use std::sync::Arc;

use crate::{
    adapters::{errors::GenericStorageError, KVStorageAdapter},
    error::{CreateConfigInstanceError, DeleteConfigInstanceError},
    model::{
        ConfigInstance, ConfigInstanceChange, ConfigInstanceRevision, RevisionReviewState,
        YakManLabel,
    },
    services::revisions,
};
use async_trait::async_trait;
use chrono::Utc;
use log::info;
use uuid::Uuid;

use super::labels::YakManLabelService;

#[async_trait]
pub trait YakManInstanceService: Sync + Send {
    async fn create_config_instance(
        &self,
        config_name: &str,
        labels: Vec<YakManLabel>,
        data: &str,
        content_type: Option<String>,
        creator_uuid: &str,
    ) -> Result<String, CreateConfigInstanceError>;

    async fn get_config_instance_metadata(
        &self,
        config_name: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, GenericStorageError>;

    async fn get_config_instance(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<ConfigInstance>, GenericStorageError>;

    async fn delete_instance(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<(), DeleteConfigInstanceError>;
}

pub struct InstanceService {
    pub adapter: Arc<dyn KVStorageAdapter>,
    pub label_service: Arc<dyn YakManLabelService>,
}

#[async_trait]
impl YakManInstanceService for InstanceService {
    async fn create_config_instance(
        &self,
        config_name: &str,
        labels: Vec<YakManLabel>,
        data: &str,
        content_type: Option<String>,
        creator_uuid: &str,
    ) -> Result<String, CreateConfigInstanceError> {
        if let Some(mut instances) = self.adapter.get_instance_metadata(config_name).await? {
            let instance = generate_instance_id();
            let revision_key: String = revisions::generate_revision_id();
            let data_key = Uuid::new_v4().to_string();
            let now = Utc::now().timestamp_millis();

            if !self.label_service.validate_labels(&labels).await? {
                return Err(CreateConfigInstanceError::InvalidLabel);
            }

            // Create new file with data
            self.adapter
                .save_instance_data(config_name, &data_key, data)
                .await?;

            // Create revision
            let revision = ConfigInstanceRevision {
                revision: String::from(&revision_key),
                data_key: String::from(&data_key),
                labels: labels,
                timestamp_ms: now,
                review_state: RevisionReviewState::Approved,
                reviewed_by_uuid: Some(creator_uuid.to_string()),
                review_timestamp_ms: Some(now),
                submitted_by_uuid: creator_uuid.to_string(),
                submit_timestamp_ms: now,
                content_type: content_type.unwrap_or(String::from("text/plain")),
            };
            self.adapter.save_revision(config_name, &revision).await?;

            // Add new instance to instances and update the instance metadata
            instances.push(ConfigInstance {
                config_name: config_name.to_string(),
                instance: instance.to_string(),
                labels: revision.labels,
                current_revision: revision.revision.clone(),
                pending_revision: None,
                revisions: vec![revision.revision.clone()],
                changelog: vec![ConfigInstanceChange {
                    timestamp_ms: now,
                    previous_revision: None,
                    new_revision: revision.revision,
                    applied_by_uuid: creator_uuid.to_string(),
                }],
            });
            self.adapter
                .save_instance_metadata(config_name, instances)
                .await?;
            info!("Update instance metadata for config: {}", config_name);

            return Ok(instance);
        }

        return Err(CreateConfigInstanceError::NoConfigFound);
    }

    async fn get_config_instance_metadata(
        &self,
        config_name: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, GenericStorageError> {
        return Ok(self.adapter.get_instance_metadata(config_name).await?);
    }

    async fn get_config_instance(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<ConfigInstance>, GenericStorageError> {
        let instances = self.get_config_instance_metadata(config_name).await?;
        return match instances {
            Some(instances) => Ok(instances.into_iter().find(|inst| inst.instance == instance)),
            None => Ok(None),
        };
    }

    async fn delete_instance(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<(), DeleteConfigInstanceError> {
        let instances = self
            .adapter
            .get_instance_metadata(config_name)
            .await?
            .ok_or(DeleteConfigInstanceError::InvalidConfig)?;

        let config_instance = instances
            .iter()
            .find(|i| i.instance == instance)
            .ok_or(DeleteConfigInstanceError::InvalidInstance)?
            .clone();

        let remaining_instances = instances
            .into_iter()
            .filter(|i| i.instance != instance)
            .collect();

        self.adapter
            .save_instance_metadata(config_name, remaining_instances)
            .await?;

        for revision in config_instance.revisions {
            if let Err(e) = self.adapter.delete_revision(config_name, &revision).await {
                log::error!("Failed to delete revision ({revision}) {e:?}");
            }
        }

        return Ok(());
    }
}

impl InstanceService {
    pub fn new(
        adapter: Arc<dyn KVStorageAdapter>,
        label_service: Arc<dyn YakManLabelService>,
    ) -> Self {
        Self {
            adapter,
            label_service,
        }
    }
}

fn generate_instance_id() -> String {
    return format!("i{}", short_sha(&Uuid::new_v4().to_string()));
}

// TODO: Move all of the short sha stuff to a shared util
/// Returns a 12 character string representation of a SHA256
fn short_sha(input: &str) -> String {
    let sha: String = sha256::digest(input);
    return sha[0..12].to_string();
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_short_sha() {
        let result = short_sha("hello world");
        assert_eq!(result, "b94d27b9934d");

        let result = short_sha("foo");
        assert_eq!(result, "2c26b46b68ff");

        let result = short_sha("bar");
        assert_eq!(result, "fcde2b2edba5");

        let result = short_sha("ade10004-41df-4bf6-88b9-d768afab674f");
        assert_eq!(result, "8146205a8d27");
    }

    #[test]
    fn test_generate_instance_id() {
        for _i in 0..10 {
            let result = generate_instance_id();
            assert_eq!(13, result.len());
            assert!(result.starts_with('i'));
        }
    }
}
