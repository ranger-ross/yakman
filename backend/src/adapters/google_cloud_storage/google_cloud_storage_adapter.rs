use std::borrow::Cow;

use super::{
    storage_types::{ApiKeysJson, ConfigJson, InstanceJson, LabelJson, UsersJson},
    GenericStorageError, KVStorageAdapter,
};
use crate::model::{
    ConfigInstance, ConfigInstanceRevision, LabelType, YakManConfig, YakManPassword,
    YakManPasswordResetLink, YakManProject, YakManSnapshotLock, YakManUser, YakManUserDetails,
};
use crate::{adapters::google_cloud_storage::storage_types::RevisionJson, model::YakManApiKey};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        copy::CopyObjectRequest,
        delete::DeleteObjectRequest,
        download::Range,
        get::GetObjectRequest,
        list::ListObjectsRequest,
        upload::{Media, UploadObjectRequest, UploadType},
    },
};
use log::error;

#[derive(Clone)]
pub struct GoogleCloudStorageAdapter {
    pub client: Client,
    pub bucket: String,
    pub root: Option<String>,
}

#[async_trait]
impl KVStorageAdapter for GoogleCloudStorageAdapter {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError> {
        let path = self.get_projects_file_path();
        let content = self.get_object(&path).await?;
        let data: Vec<YakManProject> = serde_json::from_str(&content)?;
        return Ok(data);
    }

    async fn save_projects(&self, projects: Vec<YakManProject>) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(&projects)?;
        let path = self.get_projects_file_path();
        self.put_object(&path, data).await?;
        return Ok(());
    }

    async fn get_configs(&self) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let path = self.get_configs_file_path();
        let content = self.get_object(&path).await?;
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
        self.put_object(&path, data).await?;
        Ok(())
    }

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError> {
        let path = self.get_labels_file_path();
        let content = self.get_object(&path).await?;
        let v: LabelJson = serde_json::from_str(&content)?;
        return Ok(v.labels);
    }

    async fn save_labels(&self, labels: Vec<LabelType>) -> Result<(), GenericStorageError> {
        let label_file = self.get_labels_file_path();
        let data = serde_json::to_string(&LabelJson { labels: labels })?;
        self.put_object(&label_file, data).await?;
        return Ok(());
    }

    async fn get_instance_metadata(
        &self,
        config_name: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, GenericStorageError> {
        let metadata_dir = self.get_config_instance_metadata_dir();
        let instance_file = format!("{metadata_dir}/{config_name}.json");
        if let Some(content) = self.get_object(&instance_file).await.ok() {
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

        self.put_object(&instance_file, data).await?;

        Ok(())
    }

    async fn get_revision(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<Option<ConfigInstanceRevision>, GenericStorageError> {
        let dir = self.get_instance_revisions_path();
        let path = format!("{dir}/{config_name}/{revision}");

        if let Ok(content) = self.get_object(&path).await {
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
        self.put_object(&revision_file_path, revision_data).await?;
        return Ok(());
    }

    async fn delete_revision(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<(), GenericStorageError> {
        let revisions_path = self.get_instance_revisions_path();
        let revision_file_path = format!("{revisions_path}/{config_name}/{revision}");
        self.delete_object(&revision_file_path).await?;
        return Ok(());
    }

    async fn get_instance_data(
        &self,
        config_name: &str,
        data_key: &str,
    ) -> Result<String, GenericStorageError> {
        let instance_dir = self.get_config_instance_dir();
        let instance_path = format!("{instance_dir}/{config_name}/{data_key}");
        return Ok(self.get_object(&instance_path).await?);
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
        self.put_object_with_content_type(
            &data_file_path,
            data.to_string(),
            "application/octet-stream",
        )
        .await?;
        return Ok(());
    }

    async fn initialize_yakman_storage(&self) -> Result<(), GenericStorageError> {
        let project_file = self.get_projects_file_path();
        if !self.object_exists(&project_file).await {
            self.save_projects(vec![])
                .await
                .expect("Failed to create project file");
        }

        let config_file = self.get_configs_file_path();
        if !self.object_exists(&config_file).await {
            self.save_configs(vec![])
                .await
                .expect("Failed to create config file");
        }

        let label_file = self.get_labels_file_path();
        if !self.object_exists(&label_file).await {
            self.save_labels(vec![])
                .await
                .expect("Failed to create labels file");
        }

        let user_file = self.get_user_file_path();
        if !self.object_exists(&user_file).await {
            self.save_users(vec![])
                .await
                .expect("Failed to create users file");
        }

        let api_key_file = self.get_api_key_file_path();
        if !self.object_exists(&api_key_file).await {
            self.save_api_keys(vec![])
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
        let data = self.get_object(&path).await?;
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

        if let Ok(content) = self.get_object(&path).await {
            let data: YakManUserDetails = serde_json::from_str(&content)?;
            return Ok(Some(data));
        } else {
            log::error!("Failed to load user file: {uuid}");
        }

        return Ok(None);
    }

    async fn save_user_details(
        &self,
        uuid: &str,
        details: YakManUserDetails,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_user_dir();
        let path: String = format!("{dir}/{uuid}.json");

        let data: String = serde_json::to_string(&details)?;

        self.put_object(&path, data).await?;
        return Ok(());
    }

    async fn save_users(&self, users: Vec<YakManUser>) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(&UsersJson { users: users })?;
        let data_file_path = self.get_user_file_path();
        self.put_object(&data_file_path, data).await?;
        Ok(())
    }

    async fn get_api_keys(&self) -> Result<Vec<YakManApiKey>, GenericStorageError> {
        let path = self.get_api_key_file_path();
        let data = self.get_object(&path).await?;
        let user_data: ApiKeysJson = serde_json::from_str(&data)?;
        return Ok(user_data.api_keys);
    }

    async fn save_api_keys(&self, api_keys: Vec<YakManApiKey>) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(&ApiKeysJson { api_keys: api_keys })?;
        let data_file_path = self.get_api_key_file_path();
        self.put_object(&data_file_path, data).await?;
        Ok(())
    }

    async fn save_password(
        &self,
        email_hash: &str,
        password: YakManPassword,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_password_dir();
        let path: String = format!("{dir}/{email_hash}.json");

        let data: String = serde_json::to_string(&password)?;

        self.put_object(&path, data).await?;
        return Ok(());
    }

    async fn get_password(
        &self,
        email_hash: &str,
    ) -> Result<Option<YakManPassword>, GenericStorageError> {
        let dir = self.get_password_dir();
        let path = format!("{dir}/{email_hash}.json");

        if let Ok(content) = self.get_object(&path).await {
            let data: YakManPassword = serde_json::from_str(&content)?;
            return Ok(Some(data));
        }

        return Ok(None);
    }

    async fn get_password_reset_link(
        &self,
        id: &str,
    ) -> Result<Option<YakManPasswordResetLink>, GenericStorageError> {
        let dir = self.get_password_reset_link_dir();
        let path = format!("{dir}/{id}.json");

        if let Ok(content) = self.get_object(&path).await {
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
        let path: String = format!("{dir}/{id}.json");

        let data: String = serde_json::to_string(&link)?;

        self.put_object(&path, data).await?;
        return Ok(());
    }

    async fn delete_password_reset_link(&self, id: &str) -> Result<(), GenericStorageError> {
        let dir = self.get_password_reset_link_dir();
        let path: String = format!("{dir}/{id}.json");

        self.delete_object(&path).await?;
        return Ok(());
    }

    async fn get_snapshot_lock(&self) -> Result<YakManSnapshotLock, GenericStorageError> {
        let path = self.get_snapshot_lock_file_path();
        let content = self.get_object(&path).await?;
        let data: YakManSnapshotLock = serde_json::from_str(&content)?;
        return Ok(data);
    }

    async fn save_snapshot_lock(
        &self,
        lock: &YakManSnapshotLock,
    ) -> Result<(), GenericStorageError> {
        let dir = self.get_snapshot_lock_file_path();
        let data = serde_json::to_string(&lock)?;
        self.put_object(&dir, data).await?;
        return Ok(());
    }

    async fn take_snapshot(&self, timestamp: &DateTime<Utc>) -> Result<(), GenericStorageError> {
        let snapshot_base = self.get_yakman_snapshot_dir();
        let formatted_date = timestamp.format("%Y-%m-%d-%H-%S").to_string();
        let snapshot_dir = format!("{snapshot_base}/snapshot-{formatted_date}");
        let yakman_dir = self.get_yakman_dir();

        let req = ListObjectsRequest {
            bucket: self.bucket.to_string(),
            prefix: Some(yakman_dir.clone()),
            ..Default::default()
        };

        let res = self.client.list_objects(&req).await?;

        if let Some(objects) = res.items {
            for obj in objects {
                let key = obj.name.clone();
                let new_key = key.to_string().replacen(&yakman_dir, &snapshot_dir, 1);

                let req = CopyObjectRequest {
                    source_bucket: self.bucket.to_string(),
                    source_object: key,
                    destination_bucket: self.bucket.to_string(),
                    destination_object: new_key,
                    ..Default::default()
                };
                if let Err(err) = self.client.copy_object(&req).await {
                    log::error!("Failed to copy file {err:?}");
                }
            }
        }

        Ok(())
    }
}

// Helper functions
impl GoogleCloudStorageAdapter {
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

    fn get_configs_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/configs.json");
    }

    fn get_user_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/users.json");
    }

    fn get_instance_revisions_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/instance-revisions");
    }

    fn get_api_key_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/api-keys.json");
    }

    fn get_snapshot_lock_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/snapshot-lock.json");
    }

    fn get_config_instance_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/instances");
    }

    fn get_user_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/users");
    }

    fn get_config_instance_metadata_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/instance-metadata");
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
        return self
            .put_object_with_content_type(path, data, "application/json")
            .await;
    }

    async fn put_object_with_content_type(
        &self,
        path: &str,
        data: String,
        content_type: &'static str,
    ) -> Result<(), GenericStorageError> {
        let media = Media {
            name: Cow::Owned(path.to_string()),
            content_type: Cow::Borrowed(content_type),
            content_length: None,
        };
        let upload_type = UploadType::Simple(media);
        let request = UploadObjectRequest {
            bucket: self.bucket.to_string(),
            ..Default::default()
        };
        self.client
            .clone()
            .upload_object(&request, data, &upload_type)
            .await?;

        return Ok(());
    }

    async fn delete_object(&self, path: &str) -> Result<(), GenericStorageError> {
        let request = DeleteObjectRequest {
            bucket: self.bucket.to_string(),
            object: path.to_string(),
            ..Default::default()
        };
        self.client.clone().delete_object(&request).await?;

        return Ok(());
    }

    /// Checks if a file exists in Google Cloud Storage, if an unexpected error occurs, the file is assumped to exist.
    /// This is because we use this function to check files exist at start up.
    /// To avoid accidently overriding a file on an unexpected error, we assume a file exists on an unexpected error.
    async fn object_exists(&self, key: &str) -> bool {
        let res = self
            .client
            .get_object(&GetObjectRequest {
                bucket: self.bucket.to_string(),
                object: key.to_string(),
                ..Default::default()
            })
            .await;

        return match res {
            Ok(_) => true,
            Err(e) => match e {
                google_cloud_storage::http::Error::Response(e) => e.code != 404,
                _ => true,
            },
        };
    }

    async fn get_object(&self, path: &str) -> Result<String, GenericStorageError> {
        let obj = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.to_string(),
                    object: path.to_string(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await?;

        return Ok(String::from_utf8(obj)?);
    }

    pub async fn from_env() -> Result<GoogleCloudStorageAdapter> {
        let config = ClientConfig::default().with_auth().await?;
        let client = Client::new(config);

        let bucket = std::env::var("YAKMAN_GOOGLE_CLOUD_STORAGE_BUCKET")
            .expect("YAKMAN_GOOGLE_CLOUD_STORAGE_BUCKET was not set and is required for Google Cloud Storage adapter");
        Ok(GoogleCloudStorageAdapter {
            client: client,
            bucket: bucket,
            // TODO: allow overrding from env var.
            // Reminder, truncate the trailing slash or there will be a bug
            root: None,
        })
    }
}
