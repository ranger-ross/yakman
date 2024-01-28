use std::{
    fs::{self, remove_file, File},
    io::Write,
    path::Path,
};

use async_trait::async_trait;

use crate::model::{
    ConfigInstance, ConfigInstanceRevision, LabelType, YakManApiKey, YakManConfig, YakManPassword,
    YakManPasswordResetLink, YakManProject, YakManUser, YakManUserDetails,
};
use log::{error, info};

use crate::adapters::local_file::storage_types::RevisionJson;

use super::{
    storage_types::{ApiKeysJson, ConfigJson, InstanceJson, LabelJson, UsersJson},
    GenericStorageError, KVStorageAdapter,
};

#[derive(Clone)]
pub struct LocalFileStorageAdapter {
    pub path: String,
    pub yakman_dir: Option<String>,
}

#[async_trait]
impl KVStorageAdapter for LocalFileStorageAdapter {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError> {
        let path = self.get_projects_file_path();
        let content = fs::read_to_string(path)?;
        let data: Vec<YakManProject> = serde_json::from_str(&content)?;
        return Ok(data);
    }

    async fn save_projects(&self, projects: Vec<YakManProject>) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(&projects)?;
        let path = self.get_projects_file_path();
        let mut file = File::create(&path)?;
        Write::write_all(&mut file, data.as_bytes())?;
        return Ok(());
    }

    async fn get_configs(&self) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let path = self.get_configs_file_path();
        let content = fs::read_to_string(path)?;
        let v: ConfigJson = serde_json::from_str(&content)?;
        return Ok(v.configs);
    }

    async fn get_configs_by_project_uuid(
        &self,
        project_uuid: String,
    ) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let configs = self.get_configs().await?;
        Ok(configs
            .into_iter()
            .filter(|c| c.project_uuid == project_uuid)
            .collect())
    }

    async fn save_configs(&self, configs: Vec<YakManConfig>) -> Result<(), GenericStorageError> {
        // Add config to base config file
        let data = serde_json::to_string(&ConfigJson { configs: configs })?;
        let path: String = self.get_configs_file_path();
        let mut file = File::create(&path)?;
        Write::write_all(&mut file, data.as_bytes())?;
        Ok(())
    }

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError> {
        let path = self.get_labels_file_path();
        let content = fs::read_to_string(path)?;
        let v: LabelJson = serde_json::from_str(&content)?;
        return Ok(v.labels);
    }

    async fn save_labels(&self, labels: Vec<LabelType>) -> Result<(), GenericStorageError> {
        let label_file = self.get_labels_file_path();
        let data = serde_json::to_string(&LabelJson { labels: labels })?;
        let mut file = File::create(&label_file)?;
        Write::write_all(&mut file, data.as_bytes())?;
        return Ok(());
    }

    async fn get_instance_metadata(
        &self,
        config_name: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, GenericStorageError> {
        let metadata_dir = self.get_config_instance_metadata_dir();
        let instance_file = format!("{metadata_dir}/{config_name}.json");
        if let Some(content) = fs::read_to_string(instance_file).ok() {
            let v: InstanceJson = serde_json::from_str(&content)?;
            return Ok(Some(v.instances));
        }
        return Ok(None);
    }

    async fn save_instance_metadata(
        &self,
        config_name: &str,
        instances: Vec<ConfigInstance>,
    ) -> Result<(), GenericStorageError> {
        let metadata_path = self.get_config_instance_metadata_dir();
        let instance_file = format!("{metadata_path}/{config_name}.json");
        let data = serde_json::to_string(&InstanceJson {
            instances: instances,
        })?;

        let mut file = File::create(&instance_file)?;
        Write::write_all(&mut file, data.as_bytes())?;

        Ok(())
    }

    async fn get_revision(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<Option<ConfigInstanceRevision>, GenericStorageError> {
        let dir = self.get_instance_revisions_path();
        let path = format!("{dir}/{config_name}/{revision}");

        if let Ok(content) = fs::read_to_string(&path) {
            let data: RevisionJson = serde_json::from_str(&content)?;
            return Ok(Some(data.revision));
        } else {
            error!("Failed to load revision file: {revision}");
        }

        return Ok(None);
    }

    async fn save_revision(
        &self,
        config_name: &str,
        revision: &ConfigInstanceRevision,
    ) -> Result<(), GenericStorageError> {
        let revisions_path = self.get_instance_revisions_path();
        let revision_key = &revision.revision;
        let revision_data = serde_json::to_string(&RevisionJson {
            revision: revision.clone(),
        })?;
        let revision_file_path = format!("{revisions_path}/{config_name}/{revision_key}");
        let mut revision_file = File::create(&revision_file_path)?;
        Write::write_all(&mut revision_file, revision_data.as_bytes())?;
        return Ok(());
    }

    async fn delete_revision(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<(), GenericStorageError> {
        let revisions_path = self.get_instance_revisions_path();
        remove_file(format!("{revisions_path}/{config_name}/{revision}"))?;
        return Ok(());
    }

    async fn get_instance_data(
        &self,
        config_name: &str,
        data_key: &str,
    ) -> Result<String, GenericStorageError> {
        let instance_dir = self.get_config_instance_dir();
        let instance_path = format!("{instance_dir}/{config_name}/{data_key}");
        return Ok(fs::read_to_string(instance_path)?);
    }

    async fn save_instance_data(
        &self,
        config_name: &str,
        data_key: &str,
        data: &str,
    ) -> Result<(), GenericStorageError> {
        let instance_dir = self.get_config_instance_dir();
        // Create new file with data
        let data_file_path = format!("{instance_dir}/{config_name}/{data_key}");
        let mut data_file = File::create(&data_file_path)?;
        Write::write_all(&mut data_file, data.as_bytes())?;

        return Ok(());
    }

    async fn initialize_yakman_storage(&self) -> Result<(), GenericStorageError> {
        let yakman_dir = self.get_yakman_dir();
        if !Path::new(&yakman_dir).is_dir() {
            info!("Creating {}", yakman_dir);
            fs::create_dir(&yakman_dir)
                .expect(&format!("Failed to create base dir: {}", yakman_dir));
        }

        let instance_dir = self.get_config_instance_dir();
        if !Path::new(&instance_dir).is_dir() {
            info!("Creating {}", instance_dir);
            fs::create_dir(&instance_dir)
                .expect(&format!("Failed to create instance dir: {}", instance_dir));
        }

        let revision_dir = self.get_instance_revisions_path();
        if !Path::new(&revision_dir).is_dir() {
            info!("Creating {}", revision_dir);
            fs::create_dir(&revision_dir)
                .expect(&format!("Failed to create revision dir: {}", instance_dir));
        }

        let instance_metadata_dir = self.get_config_instance_metadata_dir();
        if !Path::new(&instance_metadata_dir).is_dir() {
            info!("Creating {}", instance_metadata_dir);
            fs::create_dir(&instance_metadata_dir).expect(&format!(
                "Failed to create instance metadata dir: {}",
                instance_metadata_dir
            ));
        }

        let user_dir = self.get_user_dir();
        if !Path::new(&user_dir).is_dir() {
            info!("Creating {}", user_dir);
            fs::create_dir(&user_dir).expect(&format!(
                "Failed to create users metadata dir: {}",
                user_dir
            ));
        }

        let password_dir = self.get_password_dir();
        if !Path::new(&password_dir).is_dir() {
            log::info!("Creating {}", password_dir);
            fs::create_dir(&password_dir)
                .expect(&format!("Failed to create password dir: {}", password_dir));
        }

        let password_reset_link_dir = self.get_password_reset_link_dir();
        if !Path::new(&password_reset_link_dir).is_dir() {
            log::info!("Creating {}", password_reset_link_dir);
            fs::create_dir(&password_reset_link_dir).expect(&format!(
                "Failed to create password reset link dir: {}",
                password_reset_link_dir
            ));
        }

        let project_file = self.get_projects_file_path();
        if !Path::new(&project_file).is_file() {
            self.save_projects(vec![])
                .await
                .expect("Failed to create project file");
        }

        let config_file = self.get_configs_file_path();
        if !Path::new(&config_file).is_file() {
            self.save_configs(vec![])
                .await
                .expect("Failed to create config file");
        }

        let label_file = self.get_labels_file_path();
        if !Path::new(&label_file).is_file() {
            self.save_labels(vec![])
                .await
                .expect("Failed to create labels file");
        }

        let user_file = self.get_user_file_path();
        if !Path::new(&user_file).is_file() {
            self.save_users(vec![])
                .await
                .expect("Failed to create users file");
        }

        let api_key_file = self.get_api_key_file_path();
        if !Path::new(&api_key_file).is_file() {
            self.save_api_keys(vec![])
                .await
                .expect("Failed to create api-key file");
        }

        Ok(())
    }

    // Directory modification funcs

    async fn prepare_config_instance_storage(
        &self,
        config_name: &str,
    ) -> Result<(), GenericStorageError> {
        let config_instance_dir = self.get_config_instance_dir();
        let config_instance_path = format!("{config_instance_dir}/{config_name}");
        if !Path::new(&config_instance_path).exists() {
            fs::create_dir(&config_instance_path)?;
        }
        return Ok(());
    }

    async fn prepare_revision_instance_storage(
        &self,
        config_name: &str,
    ) -> Result<(), GenericStorageError> {
        let revision_instance_dir = self.get_instance_revisions_path();
        let revision_instance_path = format!("{revision_instance_dir}/{config_name}");
        if !Path::new(&revision_instance_path).exists() {
            fs::create_dir(&revision_instance_path)?;
        }
        return Ok(());
    }

    async fn get_users(&self) -> Result<Vec<YakManUser>, GenericStorageError> {
        let path = self.get_user_file_path();
        let data = fs::read_to_string(path)?;
        let user_data: UsersJson = serde_json::from_str(&data)?;
        return Ok(user_data.users);
    }

    async fn get_user_by_email(&self, id: &str) -> Result<Option<YakManUser>, GenericStorageError> {
        let users = self.get_users().await?;

        log::error!("{:?}", users);

        for user in users {
            if user.email == id {
                return Ok(Some(user));
            }
        }

        return Ok(None);
    }

    async fn get_user_by_uuid(
        &self,
        uuid: &str,
    ) -> Result<Option<YakManUser>, GenericStorageError> {
        let users = self.get_users().await?;

        for user in users {
            if user.uuid == uuid {
                return Ok(Some(user));
            }
        }

        return Ok(None);
    }

    async fn get_user_details(
        &self,
        uuid: &str,
    ) -> Result<Option<YakManUserDetails>, GenericStorageError> {
        let dir = self.get_user_dir();
        let path = format!("{dir}/{uuid}.json");

        if let Ok(content) = fs::read_to_string(&path) {
            let data: YakManUserDetails = serde_json::from_str(&content)?;
            return Ok(Some(data));
        } else {
            error!("Failed to load user file: {uuid}");
        }

        return Ok(None);
    }

    async fn save_user_details(
        &self,
        uuid: &str,
        details: YakManUserDetails,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_user_dir();
        let path = format!("{dir}/{uuid}.json");

        let data: String = serde_json::to_string(&details)?;

        let mut data_file = File::create(&path)?;
        Write::write_all(&mut data_file, data.as_bytes())?;

        return Ok(());
    }

    async fn save_users(&self, users: Vec<YakManUser>) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(&UsersJson { users: users })?;
        let data_file_path = self.get_user_file_path();
        let mut data_file = File::create(&data_file_path)?;
        Write::write_all(&mut data_file, data.as_bytes())?;
        Ok(())
    }

    async fn get_api_keys(&self) -> Result<Vec<YakManApiKey>, GenericStorageError> {
        let path = self.get_api_key_file_path();
        let data = fs::read_to_string(path)?;
        let data: ApiKeysJson = serde_json::from_str(&data)?;
        return Ok(data.api_keys);
    }

    async fn save_api_keys(&self, api_keys: Vec<YakManApiKey>) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(&ApiKeysJson { api_keys: api_keys })?;
        let data_file_path = self.get_api_key_file_path();
        let mut data_file = File::create(&data_file_path)?;
        Write::write_all(&mut data_file, data.as_bytes())?;
        Ok(())
    }

    async fn get_password(
        &self,
        email_hash: &str,
    ) -> Result<Option<YakManPassword>, GenericStorageError> {
        let dir = self.get_password_dir();
        let path = format!("{dir}/{email_hash}.json");

        if let Ok(content) = fs::read_to_string(&path) {
            let data: YakManPassword = serde_json::from_str(&content)?;
            return Ok(Some(data));
        }

        return Ok(None);
    }

    async fn save_password(
        &self,
        email_hash: &str,
        password: YakManPassword,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_password_dir();
        let path = format!("{dir}/{email_hash}.json");
        let data: String = serde_json::to_string(&password)?;
        let mut data_file = File::create(&path)?;
        Write::write_all(&mut data_file, data.as_bytes())?;
        return Ok(());
    }

    async fn get_password_reset_link(
        &self,
        id: &str,
    ) -> Result<Option<YakManPasswordResetLink>, GenericStorageError> {
        let dir = self.get_password_reset_link_dir();
        let path = format!("{dir}/{id}.json");

        if let Ok(content) = fs::read_to_string(&path) {
            let data: YakManPasswordResetLink = serde_json::from_str(&content)?;
            return Ok(Some(data));
        }

        return Ok(None);
    }

    async fn save_password_reset_link(
        &self,
        id: &str,
        link: YakManPasswordResetLink,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_password_reset_link_dir();
        let path = format!("{dir}/{id}.json");
        let data: String = serde_json::to_string(&link)?;
        let mut data_file = File::create(&path)?;
        Write::write_all(&mut data_file, data.as_bytes())?;
        return Ok(());
    }
}

// Helper functions
impl LocalFileStorageAdapter {
    fn get_yakman_dir(&self) -> String {
        let default_dir = String::from(".yakman");
        let yakman_dir = self.yakman_dir.as_ref().unwrap_or(&default_dir);
        return format!("{}/{yakman_dir}", self.path.as_str());
    }

    fn get_labels_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/labels.json");
    }

    fn get_projects_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/projects.json");
    }

    fn get_configs_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/configs.json");
    }

    fn get_api_key_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/api-keys.json");
    }

    fn get_user_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/users.json");
    }

    fn get_instance_revisions_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/instance-revisions");
    }

    fn get_config_instance_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/instances");
    }

    fn get_user_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/users");
    }

    fn get_password_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/passwords");
    }

    fn get_password_reset_link_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/password-reset-links");
    }

    fn get_config_instance_metadata_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/instance-metadata");
    }

    pub async fn from_env() -> LocalFileStorageAdapter {
        let directory = std::env::var("LOCAL_FILE_SYSTEM_DIRECTORY")
            .expect("LOCAL_FILE_SYSTEM_DIRECTORY was not set and is required for the LOCAL_FILE_SYSTEM adapter");

        return LocalFileStorageAdapter {
            path: directory,
            yakman_dir: None,
        };
    }
}
