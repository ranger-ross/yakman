use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::lock::Mutex;
use serde::de::DeserializeOwned;

use super::{GenericStorageError, KVStorageAdapter};
use crate::model::{
    ConfigDetails, ConfigInstanceRevision, LabelType, YakManApiKey, YakManConfig, YakManPassword,
    YakManPasswordResetLink, YakManProject, YakManProjectDetails, YakManSnapshotLock, YakManTeam,
    YakManTeamDetails, YakManUser, YakManUserDetails,
};

/// This adapter is meant for development and testing not real world use.
/// All data is lost when the service is stopped and this instance cannot be scaled horizonally.
#[derive(Clone)]
pub struct InMemoryStorageAdapter {
    pub storage: Arc<Mutex<HashMap<String, String>>>,
}

#[async_trait]
impl KVStorageAdapter for InMemoryStorageAdapter {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError> {
        let storage = self.storage.lock().await;
        let projects = storage.get(&self.get_projects_key()).unwrap();
        return Ok(serde_json::from_str(projects)?);
    }

    async fn save_projects(&self, projects: &[YakManProject]) -> Result<(), GenericStorageError> {
        self.insert(self.get_projects_key(), serde_json::to_string(projects)?)
            .await;
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
        self.insert(key, serde_json::to_string(project)?).await;
        return Ok(());
    }

    async fn delete_project_details(&self, project_id: &str) -> Result<(), GenericStorageError> {
        let key = self.get_project_key(project_id);
        self.remove(&key).await;
        return Ok(());
    }

    async fn get_configs(&self) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let storage = self.storage.lock().await;
        let configs = storage.get(&self.get_configs_key()).unwrap();
        return Ok(serde_json::from_str(configs)?);
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
        self.insert(self.get_configs_key(), serde_json::to_string(&configs)?)
            .await;
        Ok(())
    }

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError> {
        let storage = self.storage.lock().await;
        let labels = storage.get(&self.get_labels_key()).unwrap();
        return Ok(serde_json::from_str(labels)?);
    }

    async fn save_labels(&self, labels: &[LabelType]) -> Result<(), GenericStorageError> {
        self.insert(self.get_labels_key(), serde_json::to_string(&labels)?)
            .await;
        Ok(())
    }

    async fn get_instance_data(
        &self,
        config_id: &str,
        data_key: &str,
    ) -> Result<String, GenericStorageError> {
        Ok(self
            .storage
            .lock()
            .await
            .get(&self.get_data_key(config_id, data_key))
            .unwrap()
            .to_string())
    }

    async fn save_instance_data(
        &self,
        config_id: &str,
        data_key: &str,
        data: &str,
    ) -> Result<(), GenericStorageError> {
        self.insert(self.get_data_key(config_id, data_key), data.to_string())
            .await;
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
        let data = serde_json::to_string(&details)?;
        self.insert(self.get_config_details_key(config_id), data.to_string())
            .await;
        Ok(())
    }

    async fn delete_config_details(&self, config_id: &str) -> Result<(), GenericStorageError> {
        self.remove(&self.get_config_details_key(config_id)).await;
        return Ok(());
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
        let revision_key = &revision.revision;
        let data = serde_json::to_string(&revision)?;
        self.insert(
            self.get_revision_key(config_id, revision_key),
            data.to_string(),
        )
        .await;
        Ok(())
    }

    async fn delete_revision(
        &self,
        config_id: &str,
        revision: &str,
    ) -> Result<(), GenericStorageError> {
        self.remove(&self.get_revision_key(config_id, revision))
            .await;
        Ok(())
    }

    async fn prepare_config_instance_storage(&self, _: &str) -> Result<(), GenericStorageError> {
        // NOP for in memory storage
        Ok(())
    }

    async fn prepare_revision_instance_storage(&self, _: &str) -> Result<(), GenericStorageError> {
        // NOP for memory storage
        Ok(())
    }

    async fn get_users(&self) -> Result<Vec<YakManUser>, GenericStorageError> {
        let storage = self.storage.lock().await;
        let data = storage.get(&self.get_users_key()).unwrap();
        return Ok(serde_json::from_str(data)?);
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
        self.insert(key, serde_json::to_string(&details)?).await;
        return Ok(());
    }

    async fn save_users(&self, users: &[YakManUser]) -> Result<(), GenericStorageError> {
        self.insert(self.get_users_key(), serde_json::to_string(&users)?)
            .await;
        Ok(())
    }

    async fn get_api_keys(&self) -> Result<Vec<YakManApiKey>, GenericStorageError> {
        let storage = self.storage.lock().await;
        let data = storage.get(&self.get_api_keys_key()).unwrap();
        return Ok(serde_json::from_str(data)?);
    }

    async fn save_api_keys(&self, api_keys: &[YakManApiKey]) -> Result<(), GenericStorageError> {
        self.insert(self.get_api_keys_key(), serde_json::to_string(&api_keys)?)
            .await;
        Ok(())
    }

    async fn save_password(
        &self,
        email_hash: &str,
        password: &YakManPassword,
    ) -> Result<(), GenericStorageError> {
        self.insert(
            self.get_password_key(email_hash),
            serde_json::to_string(&password)?,
        )
        .await;
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
        self.insert(
            self.get_password_reset_link_key(id),
            serde_json::to_string(&link)?,
        )
        .await;
        Ok(())
    }

    async fn delete_password_reset_link(&self, id: &str) -> Result<(), GenericStorageError> {
        self.remove(&self.get_password_reset_link_key(id)).await;
        Ok(())
    }

    async fn get_teams(&self) -> Result<Vec<YakManTeam>, GenericStorageError> {
        let storage = self.storage.lock().await;
        let data = storage.get(&self.get_teams_key()).unwrap();
        return Ok(serde_json::from_str(data)?);
    }

    async fn save_teams(&self, teams: &[YakManTeam]) -> Result<(), GenericStorageError> {
        self.insert(self.get_teams_key(), serde_json::to_string(&teams)?)
            .await;
        Ok(())
    }

    async fn get_team_details(
        &self,
        team_id: &str,
    ) -> Result<Option<YakManTeamDetails>, GenericStorageError> {
        return self
            .get_optional_data(&self.get_team_details_key(team_id))
            .await;
    }

    async fn save_team_details(
        &self,
        team_id: &str,
        details: &YakManTeamDetails,
    ) -> Result<(), GenericStorageError> {
        let key = self.get_team_details_key(team_id);
        self.insert(key, serde_json::to_string(&details)?).await;
        return Ok(());
    }

    async fn delete_team_details(&self, team_id: &str) -> Result<(), GenericStorageError> {
        let key = self.get_team_details_key(team_id);
        self.remove(&key).await;
        return Ok(());
    }

    async fn get_snapshot_lock(&self) -> Result<YakManSnapshotLock, GenericStorageError> {
        let storage = self.storage.lock().await;
        let projects = storage.get(&self.get_snapshot_lock_key()).unwrap();
        return Ok(serde_json::from_str(projects)?);
    }

    async fn save_snapshot_lock(
        &self,
        lock: &YakManSnapshotLock,
    ) -> Result<(), GenericStorageError> {
        self.insert(self.get_snapshot_lock_key(), serde_json::to_string(&lock)?)
            .await;
        Ok(())
    }

    async fn take_snapshot(&self, timestamp: &DateTime<Utc>) -> Result<(), GenericStorageError> {
        let mut storage = self.storage.lock().await;

        let keys: Vec<_> = storage
            .keys()
            .filter(|k| !k.starts_with("SNAPSHOT"))
            .cloned()
            .collect();
        let keys = keys.clone();

        let snapshot_prefix = self.get_snapshot_key(timestamp);

        for key in keys {
            let value = storage.get(&key);
            if value.is_none() {
                continue;
            }
            let value = value.unwrap().clone();
            storage.insert(format!("{snapshot_prefix}_{key}"), value);
        }

        Ok(())
    }

    async fn initialize_yakman_storage(&self) -> Result<(), GenericStorageError> {
        let configs_key = self.get_configs_key();
        if !self.contains_key(&configs_key).await {
            self.save_configs(&[]).await?;
            log::info!("Initialized Configs Key");
        }

        let projects_key = self.get_projects_key();
        if !self.contains_key(&projects_key).await {
            let projects: Vec<YakManProject> = vec![];
            self.insert(projects_key, serde_json::to_string(&projects)?)
                .await;
            log::info!("Initialized Projects Key");
        }

        let labels_key = self.get_labels_key();
        if !self.contains_key(&labels_key).await {
            self.save_labels(&[]).await?;
            log::info!("Initialized Labels Key");
        }

        let users_key = self.get_users_key();
        if !self.contains_key(&users_key).await {
            let users: Vec<YakManUser> = vec![];
            self.insert(users_key, serde_json::to_string(&users)?).await;
            log::info!("Initialized Users Key");
        }

        let teams_key = self.get_teams_key();
        if !self.contains_key(&teams_key).await {
            let teams: Vec<YakManTeam> = vec![];
            self.insert(teams_key, serde_json::to_string(&teams)?).await;
            log::info!("Initialized Teams Key");
        }

        let api_key_key = self.get_api_keys_key();
        if !self.contains_key(&api_key_key).await {
            let api_keys: Vec<YakManApiKey> = vec![];
            self.insert(api_key_key, serde_json::to_string(&api_keys)?)
                .await;
            log::info!("Initialized API keys");
        }

        let snapshot_lock_key = self.get_snapshot_lock_key();
        if !self.contains_key(&snapshot_lock_key).await {
            self.save_snapshot_lock(&YakManSnapshotLock::unlocked())
                .await?;
            log::info!("Initialized snapshot log file");
        }

        Ok(())
    }
}

// Helper functions
impl InMemoryStorageAdapter {
    async fn contains_key(&self, key: &str) -> bool {
        let storage = self.storage.lock().await;
        return storage.contains_key(key);
    }

    async fn insert(&self, key: String, value: String) {
        self.storage.lock().await.insert(key, value);
    }

    async fn remove(&self, key: &str) {
        self.storage.lock().await.remove(key);
    }

    async fn get_optional_data<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, GenericStorageError> {
        let storage = self.storage.lock().await;
        let data: Option<&String> = storage.get(key);

        if let Some(data) = data {
            return Ok(serde_json::from_str(data)?);
        }

        return Ok(None);
    }

    fn get_configs_key(&self) -> String {
        "CONFIGS".to_string()
    }

    fn get_labels_key(&self) -> String {
        "LABELS".to_string()
    }

    fn get_projects_key(&self) -> String {
        "PROJECTS".to_string()
    }

    fn get_snapshot_lock_key(&self) -> String {
        "SNAPSHOT_LOCK".to_string()
    }

    fn get_users_key(&self) -> String {
        "USERS".to_string()
    }

    fn get_teams_key(&self) -> String {
        "TEAMS".to_string()
    }

    fn get_api_keys_key(&self) -> String {
        return "API_KEYS".to_string();
    }

    fn get_config_details_key(&self, config_id: &str) -> String {
        format!("CONFIG_DETAIL_{config_id}")
    }

    fn get_revision_key(&self, config_id: &str, revision: &str) -> String {
        format!("REVISION_{config_id}_{revision}")
    }

    fn get_data_key(&self, config_id: &str, data_key: &str) -> String {
        format!("CONFIG_DATA_{config_id}_{data_key}")
    }

    fn get_snapshot_key(&self, timestamp: &DateTime<Utc>) -> String {
        format!("SNAPSHOT_{}", timestamp.to_rfc3339())
    }

    fn get_project_key(&self, project_id: &str) -> String {
        format!("PROJECTS_{project_id}")
    }

    fn get_user_key(&self, user_id: &str) -> String {
        format!("USERS_{user_id}")
    }

    fn get_team_details_key(&self, team_id: &str) -> String {
        format!("TEAM_{team_id}")
    }

    fn get_password_key(&self, email_hash: &str) -> String {
        return format!("PASSWORDS_{email_hash}");
    }

    fn get_password_reset_link_key(&self, id: &str) -> String {
        return format!("PASSWORD_RESET_LINK_{id}");
    }

    pub fn new() -> InMemoryStorageAdapter {
        return InMemoryStorageAdapter {
            storage: Arc::new(Mutex::new(HashMap::new())),
        };
    }
}
