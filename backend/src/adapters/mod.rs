use yak_man_core::model::{Config, ConfigInstance, ConfigInstanceRevision, Label, LabelType};

pub mod local_file_adapter;
pub mod postgres_adapter;
pub mod redis_adapter;
mod utils;

use std::fmt;

// The base storage adapter to be able to load config from external storage

#[async_trait]
pub trait ConfigStorageAdapter: Sync + Send {
    async fn initialize_adapter(&mut self);

    async fn get_configs(&self) -> Vec<Config>;

    async fn get_labels(&self) -> Vec<LabelType>;

    async fn get_config_instance_metadata(&self, config_name: &str) -> Option<Vec<ConfigInstance>>;

    async fn get_config_data_by_labels(
        &self,
        config_name: &str,
        labels: Vec<Label>,
    ) -> Option<String>;

    async fn get_config_data(&self, config_name: &str, instance: &str) -> Option<String>;

    async fn create_config_instance(
        &self,
        config_name: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn update_config_instance(
        &self,
        config_name: &str,
        instance: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn create_config(&self, config_name: &str) -> Result<(), CreateConfigError>;

    async fn create_label(&self, label: LabelType) -> Result<(), Box<dyn std::error::Error>>;

    async fn get_instance_revisions(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Option<Vec<ConfigInstanceRevision>>;

    async fn update_instance_current_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Debug)]
pub enum CreateConfigError {
    DuplicateConfigError { name: String },
    StorageError { message: String },
}

impl std::error::Error for CreateConfigError {}

impl fmt::Display for CreateConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CreateConfigError::DuplicateConfigError { name } => {
                write!(f, "Duplicate config: `{name}`")
            }
            CreateConfigError::StorageError { message } => {
                write!(f, "Error storing config: {message}")
            }
        }
    }
}

impl CreateConfigError {
    fn duplicate_config_error(name: &str) -> CreateConfigError {
        return CreateConfigError::DuplicateConfigError {
            name: String::from(name),
        };
    }
    fn storage_error(message: &str) -> CreateConfigError {
        return CreateConfigError::StorageError {
            message: String::from(message),
        };
    }
}
