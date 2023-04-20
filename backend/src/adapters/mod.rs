use thiserror::Error;
use yak_man_core::model::{Config, ConfigInstance, ConfigInstanceRevision, Label, LabelType};

use self::errors::{ApproveRevisionError, CreateConfigError, CreateLabelError};

pub mod errors;
pub mod local_file_adapter;
pub mod postgres_adapter;
pub mod redis_adapter;
mod utils;

#[derive(Error, Debug)]
#[error("Error storing approval: ")]
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

impl From<rocket::serde::json::serde_json::Error> for GenericStorageError {
    fn from(e: rocket::serde::json::serde_json::Error) -> Self {
        GenericStorageError::new(String::from("JSON Error"), e.to_string())
    }
}

// The base storage adapter to be able to load config from external storage

#[async_trait]
pub trait ConfigStorageAdapter: Sync + Send {
    async fn initialize_adapter(&mut self);

    // async fn get_configs(&self) -> Vec<Config>;

    // async fn get_labels(&self) -> Vec<LabelType>;

    // async fn get_config_instance_metadata(&self, config_name: &str) -> Option<Vec<ConfigInstance>>;

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
    ) -> Result<(), GenericStorageError>;

    async fn update_config_instance(
        &self,
        config_name: &str,
        instance: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), GenericStorageError>;

    async fn create_config(&self, config_name: &str) -> Result<(), CreateConfigError>;

    // async fn create_label(&self, label: LabelType) -> Result<(), CreateLabelError>;

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
    ) -> Result<(), GenericStorageError>;

    async fn approve_pending_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
    ) -> Result<(), ApproveRevisionError>;
}

#[async_trait]
pub trait FileBasedStorageAdapter: Sync + Send {
    // async fn initialize_adapter(&mut self);

    async fn get_configs(&self) -> Result<Vec<Config>, GenericStorageError>;
    async fn save_configs(&self, configs: Vec<Config>) -> Result<(), GenericStorageError>;

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError>;
    async fn save_labels(&self, labels: Vec<LabelType>) -> Result<(), GenericStorageError>;

    async fn get_config_instance_metadata(&self, config_name: &str) -> Option<Vec<ConfigInstance>>;

    async fn save_config_instance_data_file(
        &self,
        config_name: &str,
        data_key: &str,
        data: &str,
    ) -> Result<(), GenericStorageError>;
    async fn get_data_by_revision(&self, config_name: &str, revision: &str) -> Option<String>;

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

    async fn save_revision_data(
        &self,
        config_name: &str,
        revision: &ConfigInstanceRevision,
    ) -> Result<(), GenericStorageError>;

    async fn create_config_instance_dir(&self, config_name: &str) -> Result<(), GenericStorageError>;
    
    async fn create_revision_instance_dir(&self, config_name: &str) -> Result<(), GenericStorageError>;

    // async fn get_config_instance_metadata(&self, config_name: &str) -> Option<Vec<ConfigInstance>>;

    // async fn get_config_data_by_labels(
    //     &self,
    //     config_name: &str,
    //     labels: Vec<Label>,
    // ) -> Option<String>;

    // async fn get_config_data(&self, config_name: &str, instance: &str) -> Option<String>;

    // async fn create_config_instance(
    //     &self,
    //     config_name: &str,
    //     labels: Vec<Label>,
    //     data: &str,
    // ) -> Result<(), GenericStorageError>;

    // async fn update_config_instance(
    //     &self,
    //     config_name: &str,
    //     instance: &str,
    //     labels: Vec<Label>,
    //     data: &str,
    // ) -> Result<(), GenericStorageError>;

    // async fn create_config(&self, config_name: &str) -> Result<(), CreateConfigError>;

    // async fn create_label(&self, label: LabelType) -> Result<(), CreateLabelError>;

    // async fn get_instance_revisions(
    //     &self,
    //     config_name: &str,
    //     instance: &str,
    // ) -> Option<Vec<ConfigInstanceRevision>>;

    // async fn update_instance_current_revision(
    //     &self,
    //     config_name: &str,
    //     instance: &str,
    //     revision: &str,
    // ) -> Result<(), GenericStorageError>;

    // async fn approve_pending_instance_revision(
    //     &self,
    //     config_name: &str,
    //     instance: &str,
    //     revision: &str,
    // ) -> Result<(), ApproveRevisionError>;
}
