use std::sync::Arc;

use crate::{
    adapters::{errors::GenericStorageError, KVStorageAdapter},
    error::{
        ApplyRevisionError, ApproveRevisionError, RollbackRevisionError, SaveConfigInstanceError,
    },
    model::{ConfigInstanceChange, ConfigInstanceRevision, RevisionReviewState, YakManLabel},
};

use async_trait::async_trait;
use chrono::Utc;
use log::info;
use uuid::Uuid;

use super::{instances::YakManInstanceService, labels::YakManLabelService};

#[async_trait]
pub trait YakManRevisionService: Sync + Send {
    /// Creates a new revision pending approval
    async fn submit_new_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        labels: Vec<YakManLabel>,
        data: &str,
        content_type: Option<String>,
        submitted_by_uuid: &str,
    ) -> Result<String, SaveConfigInstanceError>;

    async fn get_instance_revisions(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<Vec<ConfigInstanceRevision>>, GenericStorageError>;

    async fn approve_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
        approved_uuid: &str,
    ) -> Result<(), ApproveRevisionError>;

    async fn apply_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
        applied_by_uuid: &str,
    ) -> Result<(), ApplyRevisionError>;

    async fn reject_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
        rejected_by_uuid: &str,
    ) -> Result<(), ApplyRevisionError>;

    async fn rollback_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
        rollback_by_uuid: &str,
    ) -> Result<String, RollbackRevisionError>;
}

pub struct RevisionService {
    pub adapter: Arc<dyn KVStorageAdapter>,
    pub label_service: Arc<dyn YakManLabelService>,
    pub instance_service: Arc<dyn YakManInstanceService>,
}

#[async_trait]
impl YakManRevisionService for RevisionService {
    async fn submit_new_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        labels: Vec<YakManLabel>,
        data: &str,
        content_type: Option<String>,
        submitted_by_uuid: &str,
    ) -> Result<String, SaveConfigInstanceError> {
        let mut instances = self
            .adapter
            .get_instance_metadata(config_name)
            .await?
            .ok_or(SaveConfigInstanceError::InvalidConfig)?;

        let instance = instances
            .iter_mut()
            .find(|inst| inst.instance == instance)
            .ok_or(SaveConfigInstanceError::InvalidInstance)?;

        if !self.label_service.validate_labels(&labels).await? {
            return Err(SaveConfigInstanceError::InvalidLabel);
        }

        let revision_key = generate_revision_id();
        let data_key = Uuid::new_v4().to_string();

        // Create new file with data
        self.adapter
            .save_instance_data(config_name, &data_key, data)
            .await?;

        // Create revision
        let now = Utc::now().timestamp_millis();
        let revision = ConfigInstanceRevision {
            revision: String::from(&revision_key),
            data_key: String::from(&data_key),
            labels: labels,
            timestamp_ms: now,
            review_state: RevisionReviewState::Pending,
            reviewed_by_uuid: None,
            review_timestamp_ms: None,
            submitted_by_uuid: submitted_by_uuid.to_string(),
            submit_timestamp_ms: now,
            content_type: content_type.unwrap_or(String::from("text/plain")),
        };
        self.adapter.save_revision(config_name, &revision).await?;

        // Update instance data
        instance.pending_revision = Some(String::from(&revision.revision));
        instance.revisions.push(String::from(&revision.revision));

        self.adapter
            .save_instance_metadata(config_name, instances)
            .await?;

        log::info!("Updated instance metadata for config: {config_name}");

        return Ok(revision_key);
    }

    async fn get_instance_revisions(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<Vec<ConfigInstanceRevision>>, GenericStorageError> {
        let instances = match self
            .instance_service
            .get_config_instance_metadata(&config_name)
            .await?
        {
            Some(value) => value,
            None => return Ok(None),
        };

        let instance = match instances.iter().find(|inst| inst.instance == instance) {
            Some(value) => value,
            None => return Ok(None),
        };

        info!("found {} revisions", instance.revisions.len());

        let mut revisions: Vec<ConfigInstanceRevision> = vec![];

        for rev in instance.revisions.iter() {
            if let Some(revision) = self.adapter.get_revision(config_name, &rev).await? {
                revisions.push(revision);
            }
        }

        return Ok(Some(revisions));
    }

    async fn approve_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
        approved_uuid: &str,
    ) -> Result<(), ApproveRevisionError> {
        let mut metadata = match self
            .instance_service
            .get_config_instance_metadata(config_name)
            .await?
        {
            Some(metadata) => metadata,
            None => return Err(ApproveRevisionError::InvalidConfig),
        };

        let instance = match metadata.iter_mut().find(|i| i.instance == instance) {
            Some(instance) => instance,
            None => return Err(ApproveRevisionError::InvalidInstance),
        };

        // Verify instance is the pending revision
        if let Some(pending_revision) = &instance.pending_revision {
            if pending_revision != revision {
                return Err(ApproveRevisionError::InvalidRevision);
            }
        } else {
            return Err(ApproveRevisionError::InvalidRevision);
        }

        let mut revision_data = match self.adapter.get_revision(config_name, revision).await.ok() {
            Some(Some(revision_data)) => revision_data,
            None | Some(None) => return Err(ApproveRevisionError::InvalidRevision),
        };

        let now = Utc::now().timestamp_millis();
        revision_data.review_state = RevisionReviewState::Approved;
        revision_data.reviewed_by_uuid = Some(approved_uuid.to_string());
        revision_data.review_timestamp_ms = Some(now);
        self.adapter
            .save_revision(config_name, &revision_data)
            .await?;

        if !instance.revisions.contains(&String::from(revision)) {
            instance.revisions.push(String::from(revision));
        }

        self.adapter
            .save_instance_metadata(config_name, metadata)
            .await?;

        return Ok(());
    }

    async fn apply_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
        applied_by_uuid: &str,
    ) -> Result<(), ApplyRevisionError> {
        let mut metadata = match self
            .instance_service
            .get_config_instance_metadata(config_name)
            .await?
        {
            Some(metadata) => metadata,
            None => return Err(ApplyRevisionError::InvalidConfig),
        };

        let instance = match metadata.iter_mut().find(|i| i.instance == instance) {
            Some(instance) => instance,
            None => return Err(ApplyRevisionError::InvalidInstance),
        };

        // Verify instance is the pending revision
        if let Some(pending_revision) = &instance.pending_revision {
            if pending_revision != revision {
                return Err(ApplyRevisionError::InvalidRevision);
            }
        } else {
            return Err(ApplyRevisionError::InvalidRevision);
        }

        let revision_data = match self.adapter.get_revision(config_name, revision).await.ok() {
            Some(Some(revision_data)) => revision_data,
            None | Some(None) => return Err(ApplyRevisionError::InvalidRevision),
        };

        if revision_data.review_state != RevisionReviewState::Approved {
            return Err(ApplyRevisionError::NotApproved);
        }

        let now = Utc::now().timestamp_millis();
        instance.changelog.push(ConfigInstanceChange {
            timestamp_ms: now,
            previous_revision: Some(instance.current_revision.clone()),
            new_revision: String::from(revision),
            applied_by_uuid: String::from(applied_by_uuid),
        });
        instance.current_revision = String::from(revision);
        instance.pending_revision = None;
        instance.labels = revision_data.labels;

        if !instance.revisions.contains(&String::from(revision)) {
            instance.revisions.push(String::from(revision));
        }

        self.adapter
            .save_instance_metadata(config_name, metadata)
            .await?;

        return Ok(());
    }

    async fn reject_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
        rejected_by_uuid: &str,
    ) -> Result<(), ApplyRevisionError> {
        let mut metadata = match self
            .instance_service
            .get_config_instance_metadata(config_name)
            .await?
        {
            Some(metadata) => metadata,
            None => return Err(ApplyRevisionError::InvalidConfig),
        };

        let instance = match metadata.iter_mut().find(|i| i.instance == instance) {
            Some(instance) => instance,
            None => return Err(ApplyRevisionError::InvalidInstance),
        };

        let mut revision_data = match self.adapter.get_revision(config_name, revision).await.ok() {
            Some(Some(revision_data)) => revision_data,
            None | Some(None) => return Err(ApplyRevisionError::InvalidRevision),
        };

        let now = Utc::now().timestamp_millis();
        revision_data.review_state = RevisionReviewState::Rejected;
        revision_data.reviewed_by_uuid = Some(rejected_by_uuid.to_string());
        revision_data.review_timestamp_ms = Some(now);

        instance.pending_revision = None;

        if let Some(index) = instance.revisions.iter().position(|x| *x == revision) {
            instance.revisions.remove(index);
        }

        self.adapter
            .save_revision(config_name, &revision_data)
            .await?;

        self.adapter
            .save_instance_metadata(config_name, metadata)
            .await?;

        return Ok(());
    }

    async fn rollback_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
        rollback_by_uuid: &str,
    ) -> Result<String, RollbackRevisionError> {
        let mut instances = self
            .adapter
            .get_instance_metadata(config_name)
            .await?
            .ok_or(RollbackRevisionError::InvalidConfig)?;

        let instance = instances
            .iter_mut()
            .find(|inst| inst.instance == instance)
            .ok_or(RollbackRevisionError::InvalidInstance)?;

        let previous_revision = self
            .adapter
            .get_revision(&config_name, &revision)
            .await?
            .ok_or(RollbackRevisionError::InvalidRevision)?;

        let revision_key = generate_revision_id();

        // Create revision
        let now = Utc::now().timestamp_millis();
        let revision = ConfigInstanceRevision {
            revision: String::from(&revision_key),
            data_key: previous_revision.data_key,
            labels: previous_revision.labels.clone(),
            timestamp_ms: now,
            review_state: RevisionReviewState::Pending,
            reviewed_by_uuid: None,
            review_timestamp_ms: Some(now),
            submitted_by_uuid: rollback_by_uuid.to_string(),
            submit_timestamp_ms: now,
            content_type: previous_revision.content_type,
        };
        self.adapter.save_revision(config_name, &revision).await?;

        // Update instance data
        instance.pending_revision = Some(String::from(&revision.revision));
        instance.revisions.push(String::from(&revision.revision));

        self.adapter
            .save_instance_metadata(config_name, instances)
            .await?;
        log::info!("Updated instance metadata for config: {config_name}");
        return Ok(revision_key);
    }
}

impl RevisionService {
    pub fn new(
        adapter: Arc<dyn KVStorageAdapter>,
        label_service: Arc<dyn YakManLabelService>,
        instance_service: Arc<dyn YakManInstanceService>,
    ) -> Self {
        Self {
            adapter,
            label_service,
            instance_service,
        }
    }
}

pub fn generate_revision_id() -> String {
    return format!("r{}", short_sha(&Uuid::new_v4().to_string()));
}

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
    fn test_generate_revision_id() {
        for _i in 0..10 {
            let result = generate_revision_id();
            assert_eq!(13, result.len());
            assert!(result.starts_with('r'));
        }
    }
}
