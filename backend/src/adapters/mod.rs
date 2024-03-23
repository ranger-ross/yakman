use crate::model::{
    ConfigInstance, ConfigInstanceRevision, LabelType, YakManApiKey, YakManConfig, YakManPassword,
    YakManPasswordResetLink, YakManProject, YakManProjectDetails, YakManSnapshotLock, YakManUser,
    YakManUserDetails,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use self::errors::GenericStorageError;

pub mod aws_s3;
pub mod errors;
pub mod google_cloud_storage;
pub mod in_memory;
pub mod local_file;
pub mod redis;

#[async_trait]
pub trait KVStorageAdapter: Sync + Send {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError>;

    async fn save_projects(&self, projects: Vec<YakManProject>) -> Result<(), GenericStorageError>;

    async fn get_project_details(
        &self,
        project_id: &str,
    ) -> Result<Option<YakManProjectDetails>, GenericStorageError>;

    async fn save_project_details(
        &self,
        project_id: &str,
        project: YakManProjectDetails,
    ) -> Result<(), GenericStorageError>;

    async fn delete_project_details(&self, project_id: &str) -> Result<(), GenericStorageError>;

    async fn get_configs(&self) -> Result<Vec<YakManConfig>, GenericStorageError>;

    async fn get_configs_by_project_id(
        &self,
        project_id: &str,
    ) -> Result<Vec<YakManConfig>, GenericStorageError>;

    async fn save_configs(&self, configs: Vec<YakManConfig>) -> Result<(), GenericStorageError>;

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError>;

    async fn save_labels(&self, labels: Vec<LabelType>) -> Result<(), GenericStorageError>;

    async fn get_instance_metadata(
        &self,
        config_name: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, GenericStorageError>;

    async fn save_instance_metadata(
        &self,
        config_name: &str,
        instances: Vec<ConfigInstance>,
    ) -> Result<(), GenericStorageError>;

    async fn delete_instance_metadata(&self, config_name: &str) -> Result<(), GenericStorageError>;

    async fn get_instance_data(
        &self,
        config_name: &str,
        data_key: &str,
    ) -> Result<String, GenericStorageError>;

    async fn save_instance_data(
        &self,
        config_name: &str,
        data_key: &str,
        data: &str,
    ) -> Result<(), GenericStorageError>;

    async fn get_revision(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<Option<ConfigInstanceRevision>, GenericStorageError>;

    async fn save_revision(
        &self,
        config_name: &str,
        revision: &ConfigInstanceRevision,
    ) -> Result<(), GenericStorageError>;

    /// This does not delete the revision data because it might be shared across revisions.
    async fn delete_revision(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<(), GenericStorageError>;

    async fn prepare_config_instance_storage(
        &self,
        config_name: &str,
    ) -> Result<(), GenericStorageError>;

    async fn prepare_revision_instance_storage(
        &self,
        config_name: &str,
    ) -> Result<(), GenericStorageError>;

    async fn get_users(&self) -> Result<Vec<YakManUser>, GenericStorageError>;

    async fn get_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<YakManUser>, GenericStorageError>;

    /// This is for searching the main user list. It is reccomended to use `get_user_details` instead.
    async fn get_user_by_uuid(&self, uuid: &str)
        -> Result<Option<YakManUser>, GenericStorageError>;

    async fn get_user_details(
        &self,
        uuid: &str,
    ) -> Result<Option<YakManUserDetails>, GenericStorageError>;

    async fn save_user_details(
        &self,
        uuid: &str,
        details: YakManUserDetails,
    ) -> Result<(), GenericStorageError>;

    async fn save_users(&self, users: Vec<YakManUser>) -> Result<(), GenericStorageError>;

    async fn get_api_keys(&self) -> Result<Vec<YakManApiKey>, GenericStorageError>;

    async fn save_api_keys(&self, api_keys: Vec<YakManApiKey>) -> Result<(), GenericStorageError>;

    async fn get_password(
        &self,
        email_hash: &str,
    ) -> Result<Option<YakManPassword>, GenericStorageError>;

    async fn save_password(
        &self,
        email_hash: &str,
        password: YakManPassword,
    ) -> Result<(), GenericStorageError>;

    async fn get_password_reset_link(
        &self,
        id: &str,
    ) -> Result<Option<YakManPasswordResetLink>, GenericStorageError>;

    async fn save_password_reset_link(
        &self,
        id: &str,
        link: YakManPasswordResetLink,
    ) -> Result<(), GenericStorageError>;

    async fn delete_password_reset_link(&self, id: &str) -> Result<(), GenericStorageError>;

    async fn get_snapshot_lock(&self) -> Result<YakManSnapshotLock, GenericStorageError>;

    async fn save_snapshot_lock(
        &self,
        lock: &YakManSnapshotLock,
    ) -> Result<(), GenericStorageError>;

    async fn take_snapshot(&self, timestamp: &DateTime<Utc>) -> Result<(), GenericStorageError>;

    async fn initialize_yakman_storage(&self) -> Result<(), GenericStorageError>;
}
