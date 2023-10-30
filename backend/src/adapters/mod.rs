use crate::model::{
    Config, ConfigInstance, ConfigInstanceRevision, LabelType, YakManProject, YakManUser,
    YakManUserDetails,
};
use async_trait::async_trait;

use self::errors::GenericStorageError;

pub mod aws_s3;
pub mod google_cloud_storage;
pub mod errors;
pub mod local_file;
pub mod postgres;
pub mod redis;

#[async_trait]
pub trait KVStorageAdapter: Sync + Send {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError>;

    async fn save_projects(&self, projects: Vec<YakManProject>) -> Result<(), GenericStorageError>;

    async fn get_configs(&self) -> Result<Vec<Config>, GenericStorageError>;

    async fn get_configs_by_project_uuid(
        &self,
        project_uuid: String,
    ) -> Result<Vec<Config>, GenericStorageError>;

    async fn save_configs(&self, configs: Vec<Config>) -> Result<(), GenericStorageError>;

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

    async fn get_revsion(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<Option<ConfigInstanceRevision>, GenericStorageError>;

    async fn save_revision(
        &self,
        config_name: &str,
        revision: &ConfigInstanceRevision,
    ) -> Result<(), GenericStorageError>;

    async fn create_config_instance_dir(
        // TODO: Rename to "prepare" or something better?
        &self,
        config_name: &str,
    ) -> Result<(), GenericStorageError>;

    async fn create_revision_instance_dir(
        &self,
        config_name: &str,
    ) -> Result<(), GenericStorageError>;

    async fn get_users(&self) -> Result<Vec<YakManUser>, GenericStorageError>;

    async fn get_user(&self, id: &str) -> Result<Option<YakManUser>, GenericStorageError>;

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

    async fn initialize_yakman_storage(&self) -> Result<(), GenericStorageError>;
}
