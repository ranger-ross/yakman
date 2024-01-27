use crate::model::{
    ConfigInstance, ConfigInstanceRevision, LabelType, YakManApiKey, YakManConfig, YakManProject,
    YakManUser, YakManUserDetails, YakManPassword,
};
use async_trait::async_trait;

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

    async fn get_configs(&self) -> Result<Vec<YakManConfig>, GenericStorageError>;

    async fn get_configs_by_project_uuid(
        &self,
        project_uuid: String,
    ) -> Result<Vec<YakManConfig>, GenericStorageError>;

    async fn save_configs(&self, configs: Vec<YakManConfig>) -> Result<(), GenericStorageError>;

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError>;

    async fn save_labels(&self, labels: Vec<LabelType>) -> Result<(), GenericStorageError>;

    async fn get_instance_metadata(
        &self,
        config_name: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, GenericStorageError>;

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

    async fn save_instance_metadata(
        &self,
        config_name: &str,
        instances: Vec<ConfigInstance>,
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

    async fn save_password(&self, email_hash: &str, password: YakManPassword);

    async fn get_password(&self, email_hash: &str) -> Result<Option<YakManPassword>, GenericStorageError>;

    async fn initialize_yakman_storage(&self) -> Result<(), GenericStorageError>;
}
