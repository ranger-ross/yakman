use std::sync::Arc;

use crate::{
    adapters::{errors::GenericStorageError, KVStorageAdapter},
    model::{YakManUser, YakManUserDetails},
};
use async_trait::async_trait;

#[async_trait]
pub trait YakManUserService: Sync + Send {
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
}

pub struct UserService {
    pub adapter: Arc<dyn KVStorageAdapter>,
}

#[async_trait]
impl YakManUserService for UserService {
    async fn get_users(&self) -> Result<Vec<YakManUser>, GenericStorageError> {
        return self.adapter.get_users().await;
    }

    async fn get_user_by_email(&self, id: &str) -> Result<Option<YakManUser>, GenericStorageError> {
        return self.adapter.get_user_by_email(id).await;
    }

    async fn get_user_by_uuid(
        &self,
        uuid: &str,
    ) -> Result<Option<YakManUser>, GenericStorageError> {
        return self.adapter.get_user_by_uuid(uuid).await;
    }

    async fn get_user_details(
        &self,
        uuid: &str,
    ) -> Result<Option<YakManUserDetails>, GenericStorageError> {
        return self.adapter.get_user_details(uuid).await;
    }

    async fn save_user_details(
        &self,
        uuid: &str,
        details: YakManUserDetails,
    ) -> Result<(), GenericStorageError> {
        return self.adapter.save_user_details(uuid, details).await;
    }

    async fn save_users(&self, users: Vec<YakManUser>) -> Result<(), GenericStorageError> {
        return self.adapter.save_users(users).await;
    }
}

impl UserService {
    pub fn new(adapter: Arc<dyn KVStorageAdapter>) -> Self {
        Self { adapter: adapter }
    }
}
