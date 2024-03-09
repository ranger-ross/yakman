pub mod configs;
pub mod instances;
pub mod kv_storage_service;
pub mod labels;
pub mod password;
pub mod projects;
pub mod revisions;
pub mod snapshot;
pub mod users;

use crate::{
    adapters::errors::GenericStorageError,
    error::{CreateLabelError, CreatePasswordResetLinkError, ResetPasswordError},
    model::{
        LabelType, YakManApiKey, YakManPassword, YakManPublicPasswordResetLink, YakManUser,
        YakManUserDetails,
    },
};
use async_trait::async_trait;

#[async_trait]
pub trait StorageService: Sync + Send {
    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError>;

    async fn create_label(&self, label: LabelType) -> Result<(), CreateLabelError>;

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
