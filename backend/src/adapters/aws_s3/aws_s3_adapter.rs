use std::sync::Arc;

use super::{GenericStorageError, KVStorageAdapter};
use crate::model::{ConfigDetails, YakManApiKey};
use crate::model::{
    ConfigInstanceRevision, LabelType, YakManConfig, YakManPassword, YakManPasswordResetLink,
    YakManProject, YakManProjectDetails, YakManSnapshotLock, YakManTeam, YakManTeamDetails,
    YakManUser, YakManUserDetails,
};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3 as s3;
use chrono::{DateTime, Utc};
use s3::primitives::ByteStream;
use tokio::io::AsyncReadExt;

#[derive(Clone)]
pub struct AwsS3StorageAdapter {
    pub root: Option<String>,
    pub client: s3::Client,
    pub bucket: String,
}

#[async_trait]
impl KVStorageAdapter for AwsS3StorageAdapter {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError> {
        let path = self.get_projects_file_path();
        let content = self
            .get_object_as_option(&path)
            .await?
            .ok_or(AwsS3StorageAdapter::not_found())?;
        let data: Vec<YakManProject> = serde_json::from_str(&content)?;
        return Ok(data);
    }

    async fn save_projects(
        &self,
        projects: &Vec<YakManProject>,
    ) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(projects)?;
        let path = self.get_projects_file_path();
        self.put_object(&path, data).await?;
        return Ok(());
    }

    async fn get_project_details(
        &self,
        project_id: &str,
    ) -> Result<Option<YakManProjectDetails>, GenericStorageError> {
        let dir = self.get_projects_dir();
        let path = format!("{dir}/{project_id}.json");

        let Some(content) = self.get_object_as_option(&path).await? else {
            return Ok(None);
        };

        let data = serde_json::from_str(&content)?;
        return Ok(Some(data));
    }

    async fn save_project_details(
        &self,
        project_id: &str,
        project: &YakManProjectDetails,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_projects_dir();
        let path: String = format!("{dir}/{project_id}.json");
        let data: String = serde_json::to_string(project)?;
        self.put_object(&path, data).await?;
        return Ok(());
    }

    async fn delete_project_details(&self, project_id: &str) -> Result<(), GenericStorageError> {
        let dir = self.get_projects_dir();
        let path: String = format!("{dir}/{project_id}.json");
        self.delete_object(&path).await?;
        return Ok(());
    }

    async fn get_configs(&self) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let path = self.get_configs_file_path();
        let content = self
            .get_object_as_option(&path)
            .await?
            .ok_or(AwsS3StorageAdapter::not_found())?;
        return Ok(serde_json::from_str(&content)?);
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

    async fn save_configs(&self, configs: &Vec<YakManConfig>) -> Result<(), GenericStorageError> {
        // Add config to base config file
        let data = serde_json::to_string(configs)?;
        let path: String = self.get_configs_file_path();
        self.put_object(&path, data).await?;
        Ok(())
    }

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError> {
        let path = self.get_labels_file_path();
        let content = self
            .get_object_as_option(&path)
            .await?
            .ok_or(AwsS3StorageAdapter::not_found())?;
        return Ok(serde_json::from_str(&content)?);
    }

    async fn save_labels(&self, labels: &Vec<LabelType>) -> Result<(), GenericStorageError> {
        let label_file = self.get_labels_file_path();
        let data = serde_json::to_string(labels)?;
        self.put_object(&label_file, data).await?;
        return Ok(());
    }

    async fn get_config_details(
        &self,
        config_id: &str,
    ) -> Result<Option<ConfigDetails>, GenericStorageError> {
        let dir = self.get_config_details_dir();
        let instance_file = format!("{dir}/{config_id}.json");
        if let Some(content) = self.get_object_as_option(&instance_file).await? {
            return Ok(Some(serde_json::from_str(&content)?));
        }
        return Ok(None);
    }

    async fn save_config_details(
        &self,
        config_id: &str,
        details: &ConfigDetails,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_config_details_dir();
        let instance_file = format!("{dir}/{config_id}.json");
        let data = serde_json::to_string(details)?;

        self.put_object(&instance_file, data).await?;

        Ok(())
    }

    async fn delete_config_details(&self, config_id: &str) -> Result<(), GenericStorageError> {
        let dir = self.get_config_details_dir();
        let instance_file = format!("{dir}/{config_id}.json");
        self.delete_object(&instance_file).await?;
        return Ok(());
    }

    async fn get_revision(
        &self,
        config_id: &str,
        revision: &str,
    ) -> Result<Option<ConfigInstanceRevision>, GenericStorageError> {
        let dir = self.get_revisions_path();
        let path = format!("{dir}/{config_id}/{revision}");

        if let Some(content) = self.get_object_as_option(&path).await? {
            return Ok(Some(serde_json::from_str(&content)?));
        }

        return Ok(None);
    }

    async fn save_revision(
        &self,
        config_id: &str,
        revision: &ConfigInstanceRevision,
    ) -> Result<(), GenericStorageError> {
        let revisions_path = self.get_revisions_path();
        let revision_key = &revision.revision;
        let revision_data = serde_json::to_string(revision)?;
        let revision_file_path = format!("{revisions_path}/{config_id}/{revision_key}");
        self.put_object(&revision_file_path, revision_data).await?;
        return Ok(());
    }

    async fn delete_revision(
        &self,
        config_id: &str,
        revision: &str,
    ) -> Result<(), GenericStorageError> {
        let revisions_path = self.get_revisions_path();
        let revision_file_path = format!("{revisions_path}/{config_id}/{revision}");
        self.delete_object(&revision_file_path).await?;
        return Ok(());
    }

    async fn get_instance_data(
        &self,
        config_id: &str,
        data_key: &str,
    ) -> Result<String, GenericStorageError> {
        let dir = self.get_data_dir();
        let instance_path = format!("{dir}/{config_id}/{data_key}");
        return self
            .get_object_as_option(&instance_path)
            .await?
            .ok_or(AwsS3StorageAdapter::not_found());
    }

    async fn save_instance_data(
        &self,
        config_id: &str,
        data_key: &str,
        data: &str,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_data_dir();
        // Create new file with data
        let data_file_path = format!("{dir}/{config_id}/{data_key}");
        self.put_object(&data_file_path, data.to_string()).await?;
        return Ok(());
    }

    async fn initialize_yakman_storage(&self) -> Result<(), GenericStorageError> {
        let project_file = self.get_projects_file_path();
        if !self.object_exists(&project_file).await {
            self.save_projects(&vec![])
                .await
                .expect("Failed to create project file");
        }

        let config_file = self.get_configs_file_path();
        if !self.object_exists(&config_file).await {
            self.save_configs(&vec![])
                .await
                .expect("Failed to create config file");
        }

        let label_file = self.get_labels_file_path();
        if !self.object_exists(&label_file).await {
            self.save_labels(&vec![])
                .await
                .expect("Failed to create labels file");
        }

        let user_file = self.get_user_file_path();
        if !self.object_exists(&user_file).await {
            self.save_users(&vec![])
                .await
                .expect("Failed to create users file");
        }

        let api_key_file = self.get_api_key_file_path();
        if !self.object_exists(&api_key_file).await {
            self.save_api_keys(&vec![])
                .await
                .expect("Failed to create api-key file");
        }

        let snapshot_lock = self.get_snapshot_lock_file_path();
        if !self.object_exists(&snapshot_lock).await {
            self.save_snapshot_lock(&YakManSnapshotLock::unlocked())
                .await
                .expect("Failed to create snapshot lock file");
        }

        Ok(())
    }

    // Directory modification funcs

    async fn prepare_config_instance_storage(&self, _: &str) -> Result<(), GenericStorageError> {
        // NOP
        return Ok(());
    }

    async fn prepare_revision_instance_storage(&self, _: &str) -> Result<(), GenericStorageError> {
        // NOP
        return Ok(());
    }

    async fn get_users(&self) -> Result<Vec<YakManUser>, GenericStorageError> {
        let path = self.get_user_file_path();
        let data = self
            .get_object_as_option(&path)
            .await?
            .ok_or(AwsS3StorageAdapter::not_found())?;
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
        let dir = self.get_user_dir();
        let path = format!("{dir}/{user_id}.json");

        if let Some(content) = self.get_object_as_option(&path).await? {
            let data: YakManUserDetails = serde_json::from_str(&content)?;
            return Ok(Some(data));
        }

        return Ok(None);
    }

    async fn save_user_details(
        &self,
        user_id: &str,
        details: &YakManUserDetails,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_user_dir();
        let path: String = format!("{dir}/{user_id}.json");

        let data: String = serde_json::to_string(&details)?;

        self.put_object(&path, data).await?;
        return Ok(());
    }

    async fn save_users(&self, users: &Vec<YakManUser>) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(&users)?;
        let data_file_path = self.get_user_file_path();
        self.put_object(&data_file_path, data).await?;
        Ok(())
    }

    async fn get_api_keys(&self) -> Result<Vec<YakManApiKey>, GenericStorageError> {
        let path = self.get_api_key_file_path();
        let data = self
            .get_object_as_option(&path)
            .await?
            .ok_or(AwsS3StorageAdapter::not_found())?;
        return Ok(serde_json::from_str(&data)?);
    }

    async fn save_api_keys(&self, api_keys: &Vec<YakManApiKey>) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(api_keys)?;
        let data_file_path = self.get_api_key_file_path();
        self.put_object(&data_file_path, data).await?;
        Ok(())
    }

    async fn save_password(
        &self,
        email_hash: &str,
        password: &YakManPassword,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_password_dir();
        let path = format!("{dir}/{email_hash}.json");
        let data = serde_json::to_string(&password)?;
        self.put_object(&path, data).await?;
        Ok(())
    }

    async fn get_password(
        &self,
        email_hash: &str,
    ) -> Result<Option<YakManPassword>, GenericStorageError> {
        let dir = self.get_password_dir();
        let path = format!("{dir}/{email_hash}.json");
        if let Some(data) = self.get_object_as_option(&path).await? {
            let password: YakManPassword = serde_json::from_str(&data)?;
            return Ok(Some(password));
        }
        return Ok(None);
    }

    async fn get_password_reset_link(
        &self,
        id: &str,
    ) -> Result<Option<YakManPasswordResetLink>, GenericStorageError> {
        let dir = self.get_password_reset_link_dir();
        let path = format!("{dir}/{id}.json");
        if let Some(data) = self.get_object_as_option(&path).await? {
            let link: YakManPasswordResetLink = serde_json::from_str(&data)?;
            return Ok(Some(link));
        }
        return Ok(None);
    }

    async fn save_password_reset_link(
        &self,
        id: &str,
        link: &YakManPasswordResetLink,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_password_reset_link_dir();
        let path = format!("{dir}/{id}.json");
        let data = serde_json::to_string(&link)?;
        self.put_object(&path, data).await?;
        Ok(())
    }

    async fn delete_password_reset_link(&self, id: &str) -> Result<(), GenericStorageError> {
        let dir = self.get_password_reset_link_dir();
        let path = format!("{dir}/{id}.json");
        self.delete_object(&path).await?;
        Ok(())
    }

    async fn get_teams(&self) -> Result<Vec<YakManTeam>, GenericStorageError> {
        let path = self.get_teams_file_path();
        let content = self
            .get_object_as_option(&path)
            .await?
            .ok_or(AwsS3StorageAdapter::not_found())?;
        let data: Vec<_> = serde_json::from_str(&content)?;
        return Ok(data);
    }

    async fn save_teams(&self, teams: &Vec<YakManTeam>) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(&teams)?;
        let path = self.get_teams_file_path();
        self.put_object(&path, data).await?;
        return Ok(());
    }

    async fn get_team_details(
        &self,
        team_id: &str,
    ) -> Result<Option<YakManTeamDetails>, GenericStorageError> {
        let dir = self.get_teams_dir();
        let path = format!("{dir}/{team_id}.json");

        if let Some(content) = self.get_object_as_option(&path).await? {
            return Ok(Some(serde_json::from_str(&content)?));
        }

        return Ok(None);
    }

    async fn save_team_details(
        &self,
        team_id: &str,
        details: &YakManTeamDetails,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_teams_dir();
        let path: String = format!("{dir}/{team_id}.json");

        let data: String = serde_json::to_string(&details)?;

        self.put_object(&path, data).await?;
        return Ok(());
    }

    async fn delete_team_details(&self, team_id: &str) -> Result<(), GenericStorageError> {
        let dir = self.get_teams_dir();
        let path: String = format!("{dir}/{team_id}.json");
        self.delete_object(&path).await?;
        return Ok(());
    }

    async fn get_snapshot_lock(&self) -> Result<YakManSnapshotLock, GenericStorageError> {
        let path = self.get_snapshot_lock_file_path();
        let content = self
            .get_object_as_option(&path)
            .await?
            .ok_or(AwsS3StorageAdapter::not_found())?;
        let data: YakManSnapshotLock = serde_json::from_str(&content)?;
        return Ok(data);
    }

    async fn save_snapshot_lock(
        &self,
        lock: &YakManSnapshotLock,
    ) -> Result<(), GenericStorageError> {
        let path = self.get_snapshot_lock_file_path();
        let data = serde_json::to_string(&lock)?;
        self.put_object(&path, data).await?;
        Ok(())
    }

    async fn take_snapshot(&self, timestamp: &DateTime<Utc>) -> Result<(), GenericStorageError> {
        let snapshot_base = self.get_yakman_snapshot_dir();
        let formatted_date = timestamp.format("%Y-%m-%d-%H-%S").to_string();
        let snapshot_dir = format!("{snapshot_base}/snapshot-{formatted_date}");
        let yakman_dir = self.get_yakman_dir();

        let mut res = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(format!("{}/", yakman_dir))
            .max_keys(50)
            .into_paginator()
            .send();

        // Clone `self` so that the borrow checker is okay with passing a ref across multiple async context.
        // This should not be needed but the borrow checker does not know the tokio tasks will all be joined in this function
        let self_ref = Arc::new(self.clone());

        while let Some(result) = res.next().await {
            match result {
                Ok(output) => {
                    let mut handles = Vec::new();

                    for object in output.contents() {
                        if let Some(key) = object.key() {
                            let key = key.to_string();
                            let new_key = key.to_string().replacen(&yakman_dir, &snapshot_dir, 1);

                            let adapter = self_ref.clone();

                            handles.push(tokio::spawn(async move {
                                if let Err(err) = adapter.copy_object(&(key), &new_key).await {
                                    log::error!("Failed to copy file {err:?}");
                                }
                            }));
                        }
                    }

                    // Wait for all tasks to complete
                    for handle in handles {
                        handle.await.unwrap();
                    }
                }
                Err(err) => {
                    log::error!("Failed to list obects {err:?}")
                }
            }
        }

        Ok(())
    }
}

// Helper functions
impl AwsS3StorageAdapter {
    fn get_yakman_dir(&self) -> String {
        return self.get_yakman_root_dir(".yakman");
    }

    fn get_yakman_snapshot_dir(&self) -> String {
        return self.get_yakman_root_dir(".yakman-snapshot");
    }

    // Gets the path of a directory at the YakMan root
    fn get_yakman_root_dir(&self, dir: &str) -> String {
        if let Some(root) = &self.root {
            if root.is_empty() {
                return dir.to_string();
            } else {
                return format!("{root}/{dir}");
            }
        } else {
            return dir.to_string();
        }
    }

    fn get_labels_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/labels.json");
    }

    fn get_projects_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/projects.json");
    }

    fn get_teams_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/teams.json");
    }

    fn get_configs_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/configs.json");
    }

    fn get_user_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/users.json");
    }

    fn get_revisions_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/revisions");
    }

    fn get_api_key_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/api-keys.json");
    }

    fn get_snapshot_lock_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/snapshot-lock.json");
    }

    fn get_data_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/data");
    }

    fn get_projects_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/projects");
    }

    fn get_teams_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/teams");
    }

    fn get_user_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/users");
    }

    fn get_config_details_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/configs");
    }

    fn get_password_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/passwords");
    }

    fn get_password_reset_link_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/password-reset-links");
    }

    async fn put_object(&self, path: &str, data: String) -> Result<(), GenericStorageError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(path)
            .body(ByteStream::from(bytes::Bytes::from(data)))
            .send()
            .await?;
        return Ok(());
    }

    async fn delete_object(&self, path: &str) -> Result<(), GenericStorageError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await?;
        return Ok(());
    }

    /// Checks if a file exists in S3, if an unexpected error occurs, the file is assumped to exist.
    /// This is because we use this function to check files exist at start up.
    /// To avoid accidently overriding a file on an unexpected error, we assume a file exists on an unexpected error.
    async fn object_exists(&self, key: &str) -> bool {
        let x = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await;

        return match x {
            Ok(_) => true,
            Err(e) => match e {
                s3::error::SdkError::ServiceError(e) => match e.err() {
                    s3::operation::get_object::GetObjectError::NoSuchKey(_) => false,
                    _ => true,
                },
                _ => true,
            },
        };
    }

    async fn copy_object(&self, source: &str, destination: &str) -> anyhow::Result<()> {
        self.client
            .copy_object()
            .bucket(&self.bucket)
            .copy_source(format!("{}/{source}", self.bucket))
            .key(destination)
            .send()
            .await?;
        Ok(())
    }

    async fn get_object_as_option(
        &self,
        path: &str,
    ) -> Result<Option<String>, GenericStorageError> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await;

        let response = match response {
            Ok(r) => r,
            Err(e) => match &e {
                s3::error::SdkError::ServiceError(inner) => match inner.err() {
                    s3::operation::get_object::GetObjectError::NoSuchKey(_) => return Ok(None),
                    _ => return Err(e.into()),
                },
                _ => return Err(e.into()),
            },
        };

        let mut body = response.body.into_async_read();
        let mut string = String::new();
        body.read_to_string(&mut string).await?;

        return Ok(Some(string));
    }

    fn not_found() -> GenericStorageError {
        GenericStorageError::new(
            "object found".to_string(),
            "AWS adapter could not find key".to_string(),
        )
    }

    pub async fn from_env() -> AwsS3StorageAdapter {
        let config = ::aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = s3::Client::new(&config);

        let bucket = std::env::var("YAKMAN_AWS_S3_BUCKET")
            .expect("YAKMAN_AWS_S3_BUCKET was not set and is required for AWS S3 adapter");
        AwsS3StorageAdapter {
            client: client,
            bucket: bucket,
            // TODO: allow overrding from env var.
            // Reminder, truncate the trailing slash or there will be a bug
            root: None,
        }
    }
}
