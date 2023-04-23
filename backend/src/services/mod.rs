use async_trait::async_trait;
use yak_man_core::model::{Config, ConfigInstance, ConfigInstanceRevision, Label, LabelType};

use crate::adapters::errors::GenericStorageError;

use self::errors::{ApproveRevisionError, CreateConfigError, CreateLabelError};

pub mod errors;
pub mod file_based_storage_service;
mod service_utils;

#[async_trait]
pub trait StorageService: Sync + Send {
    async fn get_configs(&self) -> Result<Vec<Config>, GenericStorageError>;

    async fn get_labels(&self) -> Result<Vec<LabelType>, ()>;

    async fn create_label(&self, label: LabelType) -> Result<(), CreateLabelError>;

    async fn create_config(&self, config_name: &str) -> Result<(), CreateConfigError>;

    async fn create_config_instance(
        &self,
        config_name: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn get_config_instance_metadata(
        &self,
        config_name: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, Box<dyn std::error::Error>>;

    async fn get_config_data(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>>;

    async fn get_config_data_by_labels(
        &self,
        config_name: &str,
        labels: Vec<Label>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>>;

    async fn get_data_by_revision(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>>;

    async fn save_config_instance(
        &self,
        config_name: &str,
        instance: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn get_instance_revisions(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<Vec<ConfigInstanceRevision>>, Box<dyn std::error::Error>>;

    async fn update_instance_current_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn approve_pending_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
    ) -> Result<(), ApproveRevisionError>;

    async fn initialize_storage(&self) -> Result<(), GenericStorageError>;
}
