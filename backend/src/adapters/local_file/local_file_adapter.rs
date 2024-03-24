use std::{
    fs::{self, remove_file, File},
    io::Write,
    path::Path,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::model::{
    ConfigInstance, ConfigInstanceRevision, LabelType, YakManApiKey, YakManConfig, YakManPassword,
    YakManPasswordResetLink, YakManProject, YakManProjectDetails, YakManSnapshotLock, YakManTeam,
    YakManTeamDetails, YakManUser, YakManUserDetails,
};

use crate::adapters::local_file::storage_types::RevisionJson;

use super::{
    storage_types::{ApiKeysJson, ConfigJson, InstanceJson, LabelJson, UsersJson},
    GenericStorageError, KVStorageAdapter,
};

#[derive(Clone)]
pub struct LocalFileStorageAdapter {
    pub path: String,
    pub yakman_dir: &'static str,
    pub yakman_snapshot_dir: &'static str,
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

    async fn get_project_details(
        &self,
        project_id: &str,
    ) -> Result<Option<YakManProjectDetails>, GenericStorageError> {
        let dir = self.get_projects_dir();
        let path = format!("{dir}/{project_id}.json");

        let Ok(content) = fs::read_to_string(&path) else {
            return Ok(None);
        };

        let data: YakManProjectDetails = serde_json::from_str(&content)?;
        return Ok(Some(data));
    }

    async fn save_project_details(
        &self,
        project_id: &str,
        project: YakManProjectDetails,
    ) -> Result<(), GenericStorageError> {
        let path = self.get_projects_dir();
        let data = serde_json::to_string(&project)?;
        let revision_file_path = format!("{path}/{project_id}.json");
        let mut file = File::create(&revision_file_path)?;
        Write::write_all(&mut file, data.as_bytes())?;
        return Ok(());
    }

    async fn delete_project_details(&self, project_id: &str) -> Result<(), GenericStorageError> {
        let path = self.get_projects_dir();
        remove_file(&format!("{path}/{project_id}.json"))?;
        return Ok(());
    }

    async fn get_configs(&self) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let path = self.get_configs_file_path();
        let content = fs::read_to_string(path)?;
        let v: ConfigJson = serde_json::from_str(&content)?;
        return Ok(v.configs);
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
        config_id: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, GenericStorageError> {
        let metadata_dir = self.get_config_instance_metadata_dir();
        let instance_file = format!("{metadata_dir}/{config_id}.json");
        if let Some(content) = fs::read_to_string(instance_file).ok() {
            let v: InstanceJson = serde_json::from_str(&content)?;
            return Ok(Some(v.instances));
        }
        return Ok(None);
    }

    async fn save_instance_metadata(
        &self,
        config_id: &str,
        instances: Vec<ConfigInstance>,
    ) -> Result<(), GenericStorageError> {
        let metadata_path = self.get_config_instance_metadata_dir();
        let instance_file = format!("{metadata_path}/{config_id}.json");
        let data = serde_json::to_string(&InstanceJson {
            instances: instances,
        })?;

        let mut file = File::create(&instance_file)?;
        Write::write_all(&mut file, data.as_bytes())?;

        Ok(())
    }

    async fn delete_instance_metadata(&self, config_id: &str) -> Result<(), GenericStorageError> {
        let metadata_path = self.get_config_instance_metadata_dir();
        remove_file(&format!("{metadata_path}/{config_id}.json"))?;
        return Ok(());
    }

    async fn get_revision(
        &self,
        config_id: &str,
        revision: &str,
    ) -> Result<Option<ConfigInstanceRevision>, GenericStorageError> {
        let dir = self.get_instance_revisions_path();
        let path = format!("{dir}/{config_id}/{revision}");

        if let Ok(content) = fs::read_to_string(&path) {
            let data: RevisionJson = serde_json::from_str(&content)?;
            return Ok(Some(data.revision));
        } else {
            log::error!("Failed to load revision file: {revision}");
        }

        return Ok(None);
    }

    async fn save_revision(
        &self,
        config_id: &str,
        revision: &ConfigInstanceRevision,
    ) -> Result<(), GenericStorageError> {
        let revisions_path = self.get_instance_revisions_path();
        let revision_key = &revision.revision;
        let revision_data = serde_json::to_string(&RevisionJson {
            revision: revision.clone(),
        })?;
        let revision_file_path = format!("{revisions_path}/{config_id}/{revision_key}");
        let mut revision_file = File::create(&revision_file_path)?;
        Write::write_all(&mut revision_file, revision_data.as_bytes())?;
        return Ok(());
    }

    async fn delete_revision(
        &self,
        config_id: &str,
        revision: &str,
    ) -> Result<(), GenericStorageError> {
        let revisions_path = self.get_instance_revisions_path();
        remove_file(format!("{revisions_path}/{config_id}/{revision}"))?;
        return Ok(());
    }

    async fn get_instance_data(
        &self,
        config_id: &str,
        data_key: &str,
    ) -> Result<String, GenericStorageError> {
        let instance_dir = self.get_config_instance_dir();
        let instance_path = format!("{instance_dir}/{config_id}/{data_key}");
        return Ok(fs::read_to_string(instance_path)?);
    }

    async fn save_instance_data(
        &self,
        config_id: &str,
        data_key: &str,
        data: &str,
    ) -> Result<(), GenericStorageError> {
        let instance_dir = self.get_config_instance_dir();
        // Create new file with data
        let data_file_path = format!("{instance_dir}/{config_id}/{data_key}");
        let mut data_file = File::create(&data_file_path)?;
        Write::write_all(&mut data_file, data.as_bytes())?;

        return Ok(());
    }

    async fn initialize_yakman_storage(&self) -> Result<(), GenericStorageError> {
        let yakman_dir = self.get_yakman_dir();
        if !Path::new(&yakman_dir).is_dir() {
            log::info!("Creating {}", yakman_dir);
            fs::create_dir(&yakman_dir)
                .expect(&format!("Failed to create base dir: {}", yakman_dir));
        }

        let project_dir = self.get_projects_dir();
        if !Path::new(&project_dir).is_dir() {
            log::info!("Creating {}", project_dir);
            fs::create_dir(&project_dir)
                .expect(&format!("Failed to create project dir: {}", project_dir));
        }

        let instance_dir = self.get_config_instance_dir();
        if !Path::new(&instance_dir).is_dir() {
            log::info!("Creating {}", instance_dir);
            fs::create_dir(&instance_dir)
                .expect(&format!("Failed to create instance dir: {}", instance_dir));
        }

        let revision_dir = self.get_instance_revisions_path();
        if !Path::new(&revision_dir).is_dir() {
            log::info!("Creating {}", revision_dir);
            fs::create_dir(&revision_dir)
                .expect(&format!("Failed to create revision dir: {}", instance_dir));
        }

        let instance_dir = self.get_config_instance_metadata_dir();
        if !Path::new(&instance_dir).is_dir() {
            log::info!("Creating {}", instance_dir);
            fs::create_dir(&instance_dir)
                .expect(&format!("Failed to create instance dir: {}", instance_dir));
        }

        let user_dir = self.get_user_dir();
        if !Path::new(&user_dir).is_dir() {
            log::info!("Creating {}", user_dir);
            fs::create_dir(&user_dir).expect(&format!("Failed to create users dir: {}", user_dir));
        }

        let team_dir = self.get_teams_dir();
        if !Path::new(&team_dir).is_dir() {
            log::info!("Creating {}", team_dir);
            fs::create_dir(&team_dir).expect(&format!("Failed to create team dir: {}", team_dir));
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

        let snapshot_dir = self.get_yakman_snapshot_dir();
        if !Path::new(&snapshot_dir).is_dir() {
            log::info!("Creating {}", snapshot_dir);
            fs::create_dir(&snapshot_dir)
                .expect(&format!("Failed to create snapshot dir: {}", snapshot_dir));
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

        let team_file = self.get_team_file_path();
        if !Path::new(&team_file).is_file() {
            self.save_teams(vec![])
                .await
                .expect("Failed to create teams file");
        }

        let api_key_file = self.get_api_key_file_path();
        if !Path::new(&api_key_file).is_file() {
            self.save_api_keys(vec![])
                .await
                .expect("Failed to create api-key file");
        }

        let snapshot_lock = self.get_snapshot_lock_file_path();
        if !Path::new(&snapshot_lock).is_file() {
            self.save_snapshot_lock(&YakManSnapshotLock::unlocked())
                .await
                .expect("Failed to create snapshot lock file");
        }

        Ok(())
    }

    // Directory modification funcs

    async fn prepare_config_instance_storage(
        &self,
        config_id: &str,
    ) -> Result<(), GenericStorageError> {
        let config_instance_dir = self.get_config_instance_dir();
        let config_instance_path = format!("{config_instance_dir}/{config_id}");
        if !Path::new(&config_instance_path).exists() {
            fs::create_dir(&config_instance_path)?;
        }
        return Ok(());
    }

    async fn prepare_revision_instance_storage(
        &self,
        config_id: &str,
    ) -> Result<(), GenericStorageError> {
        let revision_instance_dir = self.get_instance_revisions_path();
        let revision_instance_path = format!("{revision_instance_dir}/{config_id}");
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
        let dir = self.get_user_dir();
        let path = format!("{dir}/{user_id}.json");

        if let Ok(content) = fs::read_to_string(&path) {
            let data: YakManUserDetails = serde_json::from_str(&content)?;
            return Ok(Some(data));
        } else {
            log::error!("Failed to load user file: {user_id}");
        }

        return Ok(None);
    }

    async fn save_user_details(
        &self,
        user_id: &str,
        details: YakManUserDetails,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_user_dir();
        let path = format!("{dir}/{user_id}.json");

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

    async fn delete_password_reset_link(&self, id: &str) -> Result<(), GenericStorageError> {
        let dir = self.get_password_reset_link_dir();
        let path = format!("{dir}/{id}.json");
        remove_file(path)?;
        Ok(())
    }

    async fn get_teams(&self) -> Result<Vec<YakManTeam>, GenericStorageError> {
        let path = self.get_team_file_path();
        let content = fs::read_to_string(path)?;
        let data: Vec<YakManTeam> = serde_json::from_str(&content)?;
        return Ok(data);
    }

    async fn save_teams(&self, teams: Vec<YakManTeam>) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(&teams)?;
        let path = self.get_team_file_path();
        let mut file = File::create(&path)?;
        Write::write_all(&mut file, data.as_bytes())?;
        return Ok(());
    }

    async fn get_team_details(
        &self,
        team_id: &str,
    ) -> Result<Option<YakManTeamDetails>, GenericStorageError> {
        let dir = self.get_teams_dir();
        let path = format!("{dir}/{team_id}.json");

        if let Ok(content) = fs::read_to_string(&path) {
            let data: YakManTeamDetails = serde_json::from_str(&content)?;
            return Ok(Some(data));
        }
        return Ok(None);
    }

    async fn save_team_details(
        &self,
        team_id: &str,
        details: YakManTeamDetails,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_teams_dir();
        let path = format!("{dir}/{team_id}.json");
        let data: String = serde_json::to_string(&details)?;
        let mut data_file = File::create(&path)?;
        Write::write_all(&mut data_file, data.as_bytes())?;
        return Ok(());
    }

    async fn delete_team_details(&self, team_id: &str) -> Result<(), GenericStorageError> {
        let path = self.get_teams_dir();
        remove_file(&format!("{path}/{team_id}.json"))?;
        return Ok(());
    }

    async fn get_snapshot_lock(&self) -> Result<YakManSnapshotLock, GenericStorageError> {
        let path = self.get_snapshot_lock_file_path();
        let data = fs::read_to_string(path)?;
        let data: YakManSnapshotLock = serde_json::from_str(&data)?;
        return Ok(data);
    }

    async fn save_snapshot_lock(
        &self,
        lock: &YakManSnapshotLock,
    ) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(&lock)?;
        let data_file_path = self.get_snapshot_lock_file_path();
        let mut data_file = File::create(&data_file_path)?;
        Write::write_all(&mut data_file, data.as_bytes())?;
        Ok(())
    }

    async fn take_snapshot(&self, timestamp: &DateTime<Utc>) -> Result<(), GenericStorageError> {
        let snapshot_base = self.get_yakman_snapshot_dir();
        let formatted_date = timestamp.format("%Y-%m-%d-%H-%S").to_string();
        let snapshot_dir = format!("{snapshot_base}/snapshot-{formatted_date}");

        let yakman_dir = self.get_yakman_dir();
        copy_dir(Path::new(&yakman_dir), Path::new(&snapshot_dir))?;
        Ok(())
    }
}

fn copy_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        if !dest.exists() {
            fs::create_dir_all(dest)?;
        }
        let entries = fs::read_dir(src)?;

        for entry in entries {
            let entry = entry?;
            let src_path = entry.path();
            let dest_path = dest.join(entry.file_name());
            copy_dir(&src_path, &dest_path)?;
        }
    } else if src.is_file() {
        fs::copy(src, dest)?;
    }
    Ok(())
}

// Helper functions
impl LocalFileStorageAdapter {
    fn get_yakman_dir(&self) -> String {
        let yakman_dir = self.yakman_dir;
        return format!("{}/{yakman_dir}", self.path.as_str());
    }

    fn get_yakman_snapshot_dir(&self) -> String {
        let yakman_snapshot_dir = self.yakman_snapshot_dir;
        return format!("{}/{yakman_snapshot_dir}", self.path.as_str());
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

    fn get_team_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/teams.json");
    }

    fn get_snapshot_lock_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/snapshot-lock.json");
    }

    fn get_instance_revisions_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/instance-revisions");
    }

    fn get_projects_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/projects");
    }

    fn get_teams_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/teams");
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
            yakman_dir: ".yakman",
            yakman_snapshot_dir: ".yakman-snapshot",
        };
    }
}
