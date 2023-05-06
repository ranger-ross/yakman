use async_trait::async_trait;
use yak_man_core::model::{
    Config, ConfigInstance, ConfigInstanceRevision, LabelType, YakManProject, YakManUser,
};

use self::errors::GenericStorageError;

pub mod errors;
pub mod local_file_adapter;
pub mod postgres_adapter;
pub mod redis_adapter;

#[async_trait]
pub trait FileBasedStorageAdapter: Sync + Send {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError>;

    async fn get_configs(&self) -> Result<Vec<Config>, GenericStorageError>;

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
        &self,
        config_name: &str,
    ) -> Result<(), GenericStorageError>;

    async fn create_revision_instance_dir(
        &self,
        config_name: &str,
    ) -> Result<(), GenericStorageError>;

    async fn get_users(&self) -> Result<Vec<YakManUser>, GenericStorageError>;

    async fn get_user(&self, id: &str) -> Result<Option<YakManUser>, GenericStorageError>;

    async fn save_users(&self, users: Vec<YakManUser>) -> Result<(), GenericStorageError>;

    async fn create_yakman_required_files(&self) -> Result<(), GenericStorageError>;
}
