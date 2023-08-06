extern crate redis;
use super::KVStorageAdapter;
use crate::adapters::errors::GenericStorageError;
use crate::model::{
    Config, ConfigInstance, ConfigInstanceRevision, LabelType, YakManProject, YakManUser,
    YakManUserDetails,
};
use async_trait::async_trait;
use log::{info, warn};
use redis::{Commands, Connection, RedisError, RedisResult};
use serde::de::DeserializeOwned;

pub struct RedisStorageAdapter {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String,
}

const REDIS_PREFIX: &str = "YAKMAN";

#[async_trait]
impl KVStorageAdapter for RedisStorageAdapter {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError> {
        let mut connection = self.open_connection()?;
        let projects: String = connection.get(self.get_projects_key())?;
        return Ok(serde_json::from_str(&projects)?);
    }

    async fn save_projects(&self, projects: Vec<YakManProject>) -> Result<(), GenericStorageError> {
        let mut connection = self.open_connection()?;
        let _: () = connection.set(self.get_projects_key(), serde_json::to_string(&projects)?)?;
        return Ok(());
    }

    async fn get_configs(&self) -> Result<Vec<Config>, GenericStorageError> {
        let mut connection = self.open_connection()?;
        let configs: String = connection.get(self.get_configs_key())?;
        return Ok(serde_json::from_str(&configs)?);
    }

    async fn get_configs_by_project_uuid(
        &self,
        project_uuid: String,
    ) -> Result<Vec<Config>, GenericStorageError> {
        let configs = self.get_configs().await?;
        Ok(configs
            .into_iter()
            .filter(|c| c.project_uuid == project_uuid)
            .collect())
    }

    async fn save_configs(&self, configs: Vec<Config>) -> Result<(), GenericStorageError> {
        let mut connection = self.open_connection()?;
        let _: () = connection.set(self.get_configs_key(), serde_json::to_string(&configs)?)?;
        Ok(())
    }

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError> {
        let mut connection = self.open_connection()?;
        let labels: String = connection.get(self.get_labels_key())?;
        return Ok(serde_json::from_str(&labels)?);
    }

    async fn save_labels(&self, labels: Vec<LabelType>) -> Result<(), GenericStorageError> {
        let mut connection = self.open_connection()?;
        let _: () = connection.set(self.get_labels_key(), serde_json::to_string(&labels)?)?;
        Ok(())
    }

    async fn get_instance_metadata(
        &self,
        config_name: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, GenericStorageError> {
        return Ok(self
            .get_optional_data(&self.get_config_metadata_key(config_name))
            .await?);
    }

    async fn get_instance_data(
        &self,
        config_name: &str,
        data_key: &str,
    ) -> Result<String, GenericStorageError> {
        let mut connection = self.open_connection()?;
        Ok(connection.get(self.get_data_key(config_name, data_key))?)
    }

    async fn save_instance_data(
        &self,
        config_name: &str,
        data_key: &str,
        data: &str,
    ) -> Result<(), GenericStorageError> {
        let mut connection = self.open_connection()?;
        let _: () = connection.set(self.get_data_key(config_name, data_key), data)?;
        Ok(())
    }

    async fn save_instance_metadata(
        &self,
        config_name: &str,
        instances: Vec<ConfigInstance>,
    ) -> Result<(), GenericStorageError> {
        let mut connection = self.open_connection()?;
        let data = serde_json::to_string(&instances)?;
        let _: () = connection.set(self.get_config_metadata_key(config_name), data)?;
        Ok(())
    }

    async fn get_revsion(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<Option<ConfigInstanceRevision>, GenericStorageError> {
        Ok(self
            .get_optional_data(&self.get_revision_key(config_name, revision))
            .await?)
    }

    async fn save_revision(
        &self,
        config_name: &str,
        revision: &ConfigInstanceRevision,
    ) -> Result<(), GenericStorageError> {
        let mut connection = self.open_connection()?;
        let revision_key = &revision.revision;
        let data = serde_json::to_string(&revision)?;
        let _: () = connection.set(self.get_revision_key(config_name, revision_key), data)?;
        Ok(())
    }

    async fn create_config_instance_dir(&self, _: &str) -> Result<(), GenericStorageError> {
        // NOP for Redis
        Ok(())
    }

    async fn create_revision_instance_dir(&self, _: &str) -> Result<(), GenericStorageError> {
        // NOP for Redis
        Ok(())
    }

    async fn get_users(&self) -> Result<Vec<YakManUser>, GenericStorageError> {
        let mut connection = self.open_connection()?;
        let data: String = connection.get(self.get_users_key())?;
        return Ok(serde_json::from_str(&data)?);
    }

    async fn get_user(&self, id: &str) -> Result<Option<YakManUser>, GenericStorageError> {
        let users = self.get_users().await?;

        for user in users {
            if user.email == id {
                return Ok(Some(user));
            }
        }

        return Ok(None);
    }

    async fn get_user_details(
        &self,
        uuid: &str,
    ) -> Result<Option<YakManUserDetails>, GenericStorageError> {
        return Ok(self.get_optional_data(&self.get_user_key(uuid)).await?);
    }

    async fn save_user_details(
        &self,
        uuid: &str,
        details: YakManUserDetails,
    ) -> Result<(), GenericStorageError> {
        let key = self.get_user_key(uuid);
        let mut connection = self.open_connection()?;
        let _: () = connection.set(key, serde_json::to_string(&details)?)?;
        return Ok(());
    }

    async fn save_users(&self, users: Vec<YakManUser>) -> Result<(), GenericStorageError> {
        let mut connection = self.open_connection()?;
        let _: () = connection.set(self.get_users_key(), serde_json::to_string(&users)?)?;
        Ok(())
    }

    async fn initialize_yakman_storage(&self) -> Result<(), GenericStorageError> {
        warn!("Redis adapter not yet setting up");

        let mut connection = self.open_connection()?;

        let configs_key = self.get_configs_key();
        if !connection.exists(&configs_key)? {
            self.save_configs(vec![]).await?;
            info!("Initialized Configs Redis Key");
        }

        let projects_key = self.get_projects_key();
        if !connection.exists(&projects_key)? {
            let projects: Vec<YakManProject> = vec![];
            let _: () = connection.set(projects_key, serde_json::to_string(&projects)?)?;
            info!("Initialized Projects Redis Key");
        }

        let labels_key = self.get_labels_key();
        if !connection.exists(&labels_key)? {
            self.save_labels(vec![]).await?;
            info!("Initialized Labels Redis Key");
        }

        let users_key = self.get_users_key();
        if !connection.exists(&users_key)? {
            let users: Vec<YakManUser> = vec![];
            let _: () = connection.set(users_key, serde_json::to_string(&users)?)?;
            info!("Initialized Users Redis Key");
        }

        Ok(())
    }
}

impl RedisStorageAdapter {
    // TODO: Connection Pooling?
    fn open_connection(&self) -> RedisResult<Connection> {
        // TODO: Handle Auth
        let connection_url: String =
            "redis://".to_string() + self.host.as_str() + ":" + self.port.to_string().as_str();
        let client = redis::Client::open(connection_url)?;
        return client.get_connection();
    }

    async fn get_optional_data<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, GenericStorageError> {
        let mut connection = self.open_connection()?;

        let data: Option<String> = connection.get(key)?;

        if let Some(data) = data {
            return Ok(serde_json::from_str(&data)?);
        }

        return Ok(None);
    }

    fn get_configs_key(&self) -> String {
        format!("{REDIS_PREFIX}_CONFIGS")
    }

    fn get_labels_key(&self) -> String {
        format!("{REDIS_PREFIX}_LABELS")
    }

    fn get_projects_key(&self) -> String {
        format!("{REDIS_PREFIX}_PROJECTS")
    }

    fn get_users_key(&self) -> String {
        format!("{REDIS_PREFIX}_USERS")
    }

    fn get_config_metadata_key(&self, config_name: &str) -> String {
        format!("{REDIS_PREFIX}_CONFIG_METADATA_{config_name}")
    }

    fn get_revision_key(&self, config_name: &str, revision: &str) -> String {
        format!("{REDIS_PREFIX}_REVISION_{config_name}_{revision}")
    }

    fn get_data_key(&self, config_name: &str, data_key: &str) -> String {
        format!("{REDIS_PREFIX}_CONFIG_DATA_{config_name}_{data_key}")
    }

    fn get_user_key(&self, uuid: &str) -> String {
        format!("{REDIS_PREFIX}_USERS_{uuid}")
    }
}

impl From<RedisError> for GenericStorageError {
    fn from(value: RedisError) -> Self {
        GenericStorageError::new(
            String::from("Redis error"),
            format!(
                "category: {}, detail: {:?}",
                value.category(),
                value.detail()
            ),
        )
    }
}
