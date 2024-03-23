pub mod kv_storage_service;
pub mod password;
pub mod snapshot;

use crate::{
    adapters::errors::GenericStorageError,
    error::{
        ApplyRevisionError, ApproveRevisionError, CreateConfigError, CreateConfigInstanceError, CreateLabelError, CreatePasswordResetLinkError, CreateProjectError, DeleteConfigError, DeleteConfigInstanceError, DeleteProjectError, ResetPasswordError, RollbackRevisionError, SaveConfigInstanceError, UpdateProjectError
    },
    model::{
        request::ProjectNotificationSettings, ConfigInstance, ConfigInstanceRevision, LabelType,
        YakManApiKey, YakManConfig, YakManLabel, YakManPassword, YakManProject,
        YakManProjectDetails, YakManPublicPasswordResetLink, YakManUser, YakManUserDetails,
    },
};
use async_trait::async_trait;

#[async_trait]
pub trait StorageService: Sync + Send {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError>;

    async fn get_project_details(
        &self,
        uuid: &str,
    ) -> Result<Option<YakManProjectDetails>, GenericStorageError>;

    async fn create_project(
        &self,
        project_name: &str,
        notification_settings: Option<ProjectNotificationSettings>,
    ) -> Result<String, CreateProjectError>;

    async fn update_project(
        &self,
        project_uuid: &str,
        project_name: &str,
        notification_settings: Option<ProjectNotificationSettings>,
    ) -> Result<(), UpdateProjectError>;

    async fn delete_project(&self, project_uuid: &str) -> Result<(), DeleteProjectError>;

    async fn get_visible_configs(
        &self,
        project_uuid: Option<String>,
    ) -> Result<Vec<YakManConfig>, GenericStorageError>;

    async fn get_config(
        &self,
        config_name: &str,
    ) -> Result<Option<YakManConfig>, GenericStorageError>;

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
        labels: Vec<YakManLabel>,
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

    async fn delete_instance(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<(), DeleteConfigInstanceError>;

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

    async fn get_users(&self) -> Result<Vec<YakManUser>, GenericStorageError>;

    async fn get_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<YakManUser>, GenericStorageError>;

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

    async fn get_api_key_by_id(
        &self,
        id: &str,
    ) -> Result<Option<YakManApiKey>, GenericStorageError>;

    async fn get_api_key_by_hash(
        &self,
        hash: &str,
    ) -> Result<Option<YakManApiKey>, GenericStorageError>;

    async fn save_api_key(&self, api_key: YakManApiKey) -> Result<(), GenericStorageError>;

    async fn delete_api_key(&self, id: &str) -> Result<(), GenericStorageError>;

    async fn get_password_by_email(
        &self,
        email: &str,
    ) -> Result<Option<YakManPassword>, GenericStorageError>;

    async fn create_password_reset_link(
        &self,
        user_uuid: &str,
    ) -> Result<YakManPublicPasswordResetLink, CreatePasswordResetLinkError>;

    async fn reset_password_with_link(
        &self,
        reset_link: YakManPublicPasswordResetLink,
        password: &str,
    ) -> Result<(), ResetPasswordError>;

    async fn validate_password_reset_link(
        &self,
        id: &str,
        user_uuid: &str,
    ) -> Result<bool, GenericStorageError>;

    async fn initialize_storage(&self) -> Result<(), GenericStorageError>;
}
