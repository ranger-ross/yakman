pub mod kv_storage_service;

use crate::{
    adapters::errors::GenericStorageError,
    error::{
        ApproveRevisionError, CreateConfigError, CreateConfigInstanceError, CreateLabelError,
        CreateProjectError, DeleteConfigError, SaveConfigInstanceError,
        UpdateConfigInstanceCurrentRevisionError,
    },
    model::{
        Config, ConfigInstance, ConfigInstanceRevision, Label, LabelType, YakManProject,
        YakManUser, YakManUserDetails,
    },
};
use async_trait::async_trait;

#[async_trait]
pub trait StorageService: Sync + Send {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError>;

    async fn create_project(&self, project_name: &str) -> Result<String, CreateProjectError>;

    async fn get_visible_configs(
        &self,
        project_uuid: Option<String>,
    ) -> Result<Vec<Config>, GenericStorageError>;

    async fn get_config(&self, config_name: &str) -> Result<Option<Config>, GenericStorageError>;

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError>;

    async fn create_label(&self, label: LabelType) -> Result<(), CreateLabelError>;

    async fn create_config(
        &self,
        config_name: &str,
        project_uuid: &str,
    ) -> Result<(), CreateConfigError>;

    async fn delete_config(&self, config_name: &str) -> Result<(), DeleteConfigError>;

    async fn create_config_instance(
        &self,
        config_name: &str,
        labels: Vec<Label>,
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

    async fn get_config_data(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<(String, String)>, GenericStorageError>;

    async fn get_data_by_revision(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<Option<(String, String)>, GenericStorageError>;

    async fn save_config_instance(
        &self,
        config_name: &str,
        instance: &str,
        labels: Vec<Label>,
        data: &str,
        content_type: Option<String>,
    ) -> Result<(), SaveConfigInstanceError>;

    async fn get_instance_revisions(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<Vec<ConfigInstanceRevision>>, GenericStorageError>;

    async fn update_instance_current_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
    ) -> Result<(), UpdateConfigInstanceCurrentRevisionError>;

    async fn approve_pending_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
        approved_uuid: &str,
    ) -> Result<(), ApproveRevisionError>;

    async fn get_users(&self) -> Result<Vec<YakManUser>, GenericStorageError>;

    async fn get_user(&self, id: &str) -> Result<Option<YakManUser>, GenericStorageError>;

    async fn get_user_details(
        &self,
        uuid: &str,
    ) -> Result<Option<YakManUserDetails>, GenericStorageError>;

    async fn save_users(&self, users: Vec<YakManUser>) -> Result<(), GenericStorageError>;

    async fn initialize_storage(&self) -> Result<(), GenericStorageError>;
}
