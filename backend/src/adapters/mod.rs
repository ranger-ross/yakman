use async_trait::async_trait;
use thiserror::Error;
use yak_man_core::model::{Config, ConfigInstance, ConfigInstanceRevision, LabelType};

pub mod errors;
pub mod local_file_adapter;
pub mod postgres_adapter;
pub mod redis_adapter;

#[derive(Error, Debug)]
#[error("Error accessing storage: {message}")]
pub struct GenericStorageError {
    pub message: String,
    pub raw_message: String,
}

impl GenericStorageError {
    fn new(message: String, raw_message: String) -> GenericStorageError {
        GenericStorageError {
            message: message,
            raw_message: raw_message,
        }
    }
}

impl From<std::io::Error> for GenericStorageError {
    fn from(e: std::io::Error) -> Self {
        GenericStorageError::new(String::from("IO Error"), e.to_string())
    }
}

impl From<serde_json::Error> for GenericStorageError {
    fn from(e: serde_json::Error) -> Self {
        GenericStorageError::new(String::from("JSON Error"), e.to_string())
    }
}

#[async_trait]
pub trait FileBasedStorageAdapter: Sync + Send {
    async fn get_configs(&self) -> Result<Vec<Config>, GenericStorageError>;

    async fn save_configs(&self, configs: Vec<Config>) -> Result<(), GenericStorageError>;

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError>;

    async fn save_labels(&self, labels: Vec<LabelType>) -> Result<(), GenericStorageError>;

    async fn get_instance_metadata(&self, config_name: &str) -> Result<Option<Vec<ConfigInstance>>, GenericStorageError>;

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
    ) -> Option<ConfigInstanceRevision>;

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

    async fn create_yakman_required_files(&self) -> Result<(), GenericStorageError>;
}
