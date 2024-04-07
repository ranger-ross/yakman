extern crate redis;
use std::env;

use super::KVStorageAdapter;
use crate::adapters::errors::GenericStorageError;
use crate::model::{
    ConfigDetails, ConfigInstanceRevision, LabelType, YakManApiKey, YakManConfig, YakManPassword,
    YakManPasswordResetLink, YakManProject, YakManProjectDetails, YakManSnapshotLock, YakManTeam,
    YakManTeamDetails, YakManUser, YakManUserDetails,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::info;
use r2d2::PooledConnection;
use redis::{Commands, RedisError};
use serde::de::DeserializeOwned;

pub struct RedisStorageAdapter {
    pub host: String,
    pub port: i32,
    pub username: Option<String>,
    pub password: Option<String>,
    pub connection_pool: r2d2::Pool<redis::Client>,
}

const REDIS_PREFIX: &str = "YAKMAN_DATA";
const SNAPSHOT_PREFIX: &str = "YAKMAN_SNAPSHOT";

#[async_trait]
impl KVStorageAdapter for RedisStorageAdapter {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError> {
        let mut connection = self.get_connection()?;
        let projects: String = connection.get(self.get_projects_key())?;
        return Ok(serde_json::from_str(&projects)?);
    }

    async fn save_projects(&self, projects: &[YakManProject]) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let _: () = connection.set(self.get_projects_key(), serde_json::to_string(projects)?)?;
        return Ok(());
    }

    async fn get_project_details(
        &self,
        project_id: &str,
    ) -> Result<Option<YakManProjectDetails>, GenericStorageError> {
        return self
            .get_optional_data(&self.get_project_key(project_id))
            .await;
    }

    async fn save_project_details(
        &self,
        project_id: &str,
        project: &YakManProjectDetails,
    ) -> Result<(), GenericStorageError> {
        let key = self.get_project_key(project_id);
        let mut connection = self.get_connection()?;
        let _: () = connection.set(key, serde_json::to_string(project)?)?;
        return Ok(());
    }

    async fn delete_project_details(&self, project_id: &str) -> Result<(), GenericStorageError> {
        let key = self.get_project_key(project_id);
        let mut connection = self.get_connection()?;
        let _: () = connection.del(&key)?;
        Ok(())
    }

    async fn get_configs(&self) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let mut connection = self.get_connection()?;
        let configs: String = connection.get(self.get_configs_key())?;
        return Ok(serde_json::from_str(&configs)?);
    }

    async fn get_configs_by_project_id(
        &self,
        project_id: &str,
    ) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let configs = self.get_configs().await?;
        Ok(configs
            .into_iter()
            .filter(|c| c.project_id == project_id)
            .collect())
    }

    async fn save_configs(&self, configs: &[YakManConfig]) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let _: () = connection.set(self.get_configs_key(), serde_json::to_string(&configs)?)?;
        Ok(())
    }

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError> {
        let mut connection = self.get_connection()?;
        let labels: String = connection.get(self.get_labels_key())?;
        return Ok(serde_json::from_str(&labels)?);
    }

    async fn save_labels(&self, labels: &[LabelType]) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let _: () = connection.set(self.get_labels_key(), serde_json::to_string(&labels)?)?;
        Ok(())
    }

    async fn get_instance_data(
        &self,
        config_id: &str,
        data_key: &str,
    ) -> Result<String, GenericStorageError> {
        let mut connection = self.get_connection()?;
        Ok(connection.get(self.get_data_key(config_id, data_key))?)
    }

    async fn save_instance_data(
        &self,
        config_id: &str,
        data_key: &str,
        data: &str,
    ) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let _: () = connection.set(self.get_data_key(config_id, data_key), data)?;
        Ok(())
    }

    async fn get_config_details(
        &self,
        config_id: &str,
    ) -> Result<Option<ConfigDetails>, GenericStorageError> {
        return self
            .get_optional_data(&self.get_config_details_key(config_id))
            .await;
    }

    async fn save_config_details(
        &self,
        config_id: &str,
        details: &ConfigDetails,
    ) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let data = serde_json::to_string(&details)?;
        let _: () = connection.set(self.get_config_details_key(config_id), data)?;
        Ok(())
    }

    async fn delete_config_details(&self, config_id: &str) -> Result<(), GenericStorageError> {
        let key = self.get_config_details_key(config_id);
        let mut connection = self.get_connection()?;
        let _: () = connection.del(&key)?;
        Ok(())
    }

    async fn get_revision(
        &self,
        config_id: &str,
        revision: &str,
    ) -> Result<Option<ConfigInstanceRevision>, GenericStorageError> {
        Ok(self
            .get_optional_data(&self.get_revision_key(config_id, revision))
            .await?)
    }

    async fn save_revision(
        &self,
        config_id: &str,
        revision: &ConfigInstanceRevision,
    ) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let revision_key = &revision.revision;
        let data = serde_json::to_string(&revision)?;
        let _: () = connection.set(self.get_revision_key(config_id, revision_key), data)?;
        Ok(())
    }

    async fn delete_revision(
        &self,
        config_id: &str,
        revision: &str,
    ) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let _: () = connection.del(&self.get_revision_key(config_id, revision))?;
        Ok(())
    }

    async fn prepare_config_instance_storage(&self, _: &str) -> Result<(), GenericStorageError> {
        // NOP for Redis
        Ok(())
    }

    async fn prepare_revision_instance_storage(&self, _: &str) -> Result<(), GenericStorageError> {
        // NOP for Redis
        Ok(())
    }

    async fn get_users(&self) -> Result<Vec<YakManUser>, GenericStorageError> {
        let mut connection = self.get_connection()?;
        let data: String = connection.get(self.get_users_key())?;
        return Ok(serde_json::from_str(&data)?);
    }

    async fn get_user_by_email(&self, id: &str) -> Result<Option<YakManUser>, GenericStorageError> {
        let users = self.get_users().await?;

        for user in users {
            if user.email == id {
                return Ok(Some(user));
            }
        }

        return Ok(None);
    }

    async fn get_user_by_id(
        &self,
        user_id: &str,
    ) -> Result<Option<YakManUser>, GenericStorageError> {
        let users = self.get_users().await?;

        for user in users {
            if user.id == user_id {
                return Ok(Some(user));
            }
        }

        return Ok(None);
    }

    async fn get_user_details(
        &self,
        user_id: &str,
    ) -> Result<Option<YakManUserDetails>, GenericStorageError> {
        return self.get_optional_data(&self.get_user_key(user_id)).await;
    }

    async fn save_user_details(
        &self,
        user_id: &str,
        details: &YakManUserDetails,
    ) -> Result<(), GenericStorageError> {
        let key = self.get_user_key(user_id);
        let mut connection = self.get_connection()?;
        let _: () = connection.set(key, serde_json::to_string(&details)?)?;
        return Ok(());
    }

    async fn save_users(&self, users: &[YakManUser]) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let _: () = connection.set(self.get_users_key(), serde_json::to_string(&users)?)?;
        Ok(())
    }

    async fn get_api_keys(&self) -> Result<Vec<YakManApiKey>, GenericStorageError> {
        let mut connection = self.get_connection()?;
        let data: String = connection.get(self.get_api_keys_key())?;
        return Ok(serde_json::from_str(&data)?);
    }

    async fn save_api_keys(&self, api_keys: &[YakManApiKey]) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let _: () = connection.set(self.get_api_keys_key(), serde_json::to_string(&api_keys)?)?;
        Ok(())
    }

    async fn save_password(
        &self,
        email_hash: &str,
        password: &YakManPassword,
    ) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let _: () = connection.set(
            self.get_password_key(email_hash),
            serde_json::to_string(&password)?,
        )?;
        Ok(())
    }

    async fn get_password(
        &self,
        email_hash: &str,
    ) -> Result<Option<YakManPassword>, GenericStorageError> {
        return self
            .get_optional_data(&self.get_password_key(email_hash))
            .await;
    }

    async fn get_password_reset_link(
        &self,
        id: &str,
    ) -> Result<Option<YakManPasswordResetLink>, GenericStorageError> {
        return self
            .get_optional_data(&self.get_password_reset_link_key(id))
            .await;
    }

    async fn save_password_reset_link(
        &self,
        id: &str,
        link: &YakManPasswordResetLink,
    ) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let _: () = connection.set(
            self.get_password_reset_link_key(id),
            serde_json::to_string(&link)?,
        )?;
        Ok(())
    }

    async fn delete_password_reset_link(&self, id: &str) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let _: () = connection.del(self.get_password_reset_link_key(id))?;
        Ok(())
    }

    async fn get_teams(&self) -> Result<Vec<YakManTeam>, GenericStorageError> {
        let mut connection = self.get_connection()?;
        let projects: String = connection.get(self.get_teams_key())?;
        return Ok(serde_json::from_str(&projects)?);
    }

    async fn save_teams(&self, teams: &[YakManTeam]) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let _: () = connection.set(self.get_teams_key(), serde_json::to_string(&teams)?)?;
        return Ok(());
    }

    async fn get_team_details(
        &self,
        team_id: &str,
    ) -> Result<Option<YakManTeamDetails>, GenericStorageError> {
        return self.get_optional_data(&self.get_team_key(team_id)).await;
    }

    async fn save_team_details(
        &self,
        team_id: &str,
        details: &YakManTeamDetails,
    ) -> Result<(), GenericStorageError> {
        let key = self.get_team_key(team_id);
        let mut connection = self.get_connection()?;
        let _: () = connection.set(key, serde_json::to_string(&details)?)?;
        return Ok(());
    }

    async fn delete_team_details(&self, team_id: &str) -> Result<(), GenericStorageError> {
        let key = self.get_team_key(team_id);
        let mut connection = self.get_connection()?;
        let _: () = connection.del(&key)?;
        Ok(())
    }

    async fn get_snapshot_lock(&self) -> Result<YakManSnapshotLock, GenericStorageError> {
        return self
            .get_optional_data(&self.get_snapshot_lock_key())
            .await?
            .ok_or(GenericStorageError::new(
                "Lockfile not found".to_string(),
                "Lockfile not found".to_string(),
            ));
    }

    async fn save_snapshot_lock(
        &self,
        lock: &YakManSnapshotLock,
    ) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let _: () = connection.set(self.get_snapshot_lock_key(), serde_json::to_string(&lock)?)?;
        Ok(())
    }

    async fn take_snapshot(&self, timestamp: &DateTime<Utc>) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;
        let keys: Vec<String> = connection.keys(format!("{REDIS_PREFIX}*"))?;

        let formatted_date = timestamp.format("%Y%m%d%H%S").to_string();
        let snapshot_prefix = format!("{SNAPSHOT_PREFIX}_{formatted_date}");

        for key in keys {
            let new_key = key.to_string().replacen(REDIS_PREFIX, &snapshot_prefix, 1);
            if let Err(err) = redis::cmd("COPY")
                .arg(&key)
                .arg(new_key)
                .query::<()>(&mut connection)
            {
                log::error!("Failed to copy key {key}, Err: {err:?}");
            }
        }

        return Ok(());
    }

    async fn initialize_yakman_storage(&self) -> Result<(), GenericStorageError> {
        let mut connection = self.get_connection()?;

        let configs_key = self.get_configs_key();
        if !connection.exists(&configs_key)? {
            self.save_configs(&[]).await?;
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
            self.save_labels(&[]).await?;
            info!("Initialized Labels Redis Key");
        }

        let users_key = self.get_users_key();
        if !connection.exists(&users_key)? {
            let users: Vec<YakManUser> = vec![];
            let _: () = connection.set(users_key, serde_json::to_string(&users)?)?;
            info!("Initialized Users Redis Key");
        }

        let api_key_key = self.get_users_key();
        if !connection.exists(&api_key_key)? {
            let api_keys: Vec<YakManApiKey> = vec![];
            let _: () = connection.set(api_key_key, serde_json::to_string(&api_keys)?)?;
            info!("Initialized ApiKeys Redis Key");
        }

        let snapshot_lock_key = self.get_snapshot_lock_key();
        if !connection.exists(&snapshot_lock_key)? {
            let lock = YakManSnapshotLock::unlocked();
            let _: () = connection.set(snapshot_lock_key, serde_json::to_string(&lock)?)?;
            info!("Initialized snapshot lock Redis Key");
        }

        Ok(())
    }
}

const DEFAULT_REDIS_PORT: i32 = 6379;

impl RedisStorageAdapter {
    pub async fn from_env() -> Result<RedisStorageAdapter> {
        let host = env::var("YAKMAN_REDIS_HOST")
            .expect("YAKMAN_REDIS_HOST was not set and is required by the Redis adapter");

        let port: i32 = env::var("YAKMAN_REDIS_PORT")
            .map(|v| v.parse::<i32>().unwrap_or(DEFAULT_REDIS_PORT))
            .unwrap_or(DEFAULT_REDIS_PORT);

        let username = env::var("YAKMAN_REDIS_USERNAME").ok();
        let password = env::var("YAKMAN_REDIS_PASSWORD").ok();

        let connection_url: String =
            Self::create_connection_url(&host, port, username.as_deref(), password.as_deref());

        let client = redis::Client::open(connection_url)?;

        let pool: r2d2::Pool<redis::Client> = r2d2::Pool::builder().build(client)?;

        return Ok(RedisStorageAdapter {
            host: host,
            port: port,
            username: username,
            password: password,
            connection_pool: pool,
        });
    }

    fn get_connection(&self) -> Result<PooledConnection<redis::Client>, GenericStorageError> {
        return Ok(self.connection_pool.get()?);
    }

    fn create_connection_url(
        host: &str,
        port: i32,
        username: Option<&str>,
        password: Option<&str>,
    ) -> String {
        let auth = match (&username, &password) {
            (Some(u), Some(p)) => format!("{}:{}@", u, p),
            (Some(u), None) => format!("{}@", u),
            (None, Some(p)) => format!(":{}@", p),
            (None, None) => String::new(),
        };

        return "redis://".to_string() + &auth + host + ":" + port.to_string().as_str();
    }

    async fn get_optional_data<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, GenericStorageError> {
        let mut connection = self.get_connection()?;

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

    fn get_teams_key(&self) -> String {
        format!("{REDIS_PREFIX}_TEAMS")
    }

    fn get_users_key(&self) -> String {
        format!("{REDIS_PREFIX}_USERS")
    }

    fn get_api_keys_key(&self) -> String {
        return format!("{REDIS_PREFIX}_API_KEYS");
    }

    fn get_snapshot_lock_key(&self) -> String {
        return format!("{REDIS_PREFIX}_SNAPSHOT_LOCK");
    }

    fn get_config_details_key(&self, config_id: &str) -> String {
        format!("{REDIS_PREFIX}_CONFIG_DETAILS_{config_id}")
    }

    fn get_revision_key(&self, config_id: &str, revision: &str) -> String {
        format!("{REDIS_PREFIX}_REVISION_{config_id}_{revision}")
    }

    fn get_data_key(&self, config_id: &str, data_key: &str) -> String {
        format!("{REDIS_PREFIX}_CONFIG_DATA_{config_id}_{data_key}")
    }

    fn get_project_key(&self, project_id: &str) -> String {
        format!("{REDIS_PREFIX}_PROJECTS_{project_id}")
    }

    fn get_team_key(&self, team_id: &str) -> String {
        format!("{REDIS_PREFIX}_TEAMS_{team_id}")
    }

    fn get_user_key(&self, user_id: &str) -> String {
        format!("{REDIS_PREFIX}_USERS_{user_id}")
    }

    fn get_password_key(&self, email_hash: &str) -> String {
        return format!("{REDIS_PREFIX}_PASSWORDS_{email_hash}");
    }

    fn get_password_reset_link_key(&self, id: &str) -> String {
        return format!("{REDIS_PREFIX}_PASSWORD_RESET_LINK_{id}");
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

impl From<r2d2::Error> for GenericStorageError {
    fn from(value: r2d2::Error) -> Self {
        GenericStorageError {
            message: String::from("Redis connection pool error"),
            raw_message: value.to_string(),
        }
    }
}
