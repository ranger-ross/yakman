use super::{
    storage_types::{ApiKeysJson, ConfigJson, InstanceJson, LabelJson, UsersJson},
    GenericStorageError, KVStorageAdapter,
};
use crate::model::{
    ConfigInstance, ConfigInstanceRevision, LabelType, YakManConfig, YakManPassword, YakManPasswordResetLink, YakManProject, YakManSnapshotLock, YakManUser, YakManUserDetails
};
use crate::{adapters::aws_s3::storage_types::RevisionJson, model::YakManApiKey};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3 as s3;
use s3::primitives::ByteStream;
use tokio::io::AsyncReadExt;

#[derive(Clone)]
pub struct AwsS3StorageAdapter {
    pub yakman_dir: Option<String>,
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

    async fn save_projects(&self, projects: Vec<YakManProject>) -> Result<(), GenericStorageError> {
        let data = serde_json::to_string(&projects)?;
        let path = self.get_projects_file_path();
        self.put_object(&path, data).await?;
        return Ok(());
    }

    async fn get_configs(&self) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let path = self.get_configs_file_path();
        let content = self
            .get_object_as_option(&path)
            .await?
            .ok_or(AwsS3StorageAdapter::not_found())?;
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
        let content = self
            .get_object_as_option(&path)
            .await?
            .ok_or(AwsS3StorageAdapter::not_found())?;
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
        if let Some(content) = self.get_object_as_option(&instance_file).await? {
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

        if let Some(content) = self.get_object_as_option(&path).await? {
            let data: RevisionJson = serde_json::from_str(&content)?;
            return Ok(Some(data.revision));
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
        return Ok(self
            .get_object_as_option(&instance_path)
            .await?
            .ok_or(AwsS3StorageAdapter::not_found())?);
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
        self.put_object(&data_file_path, data.to_string()).await?;
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

        if let Some(content) = self.get_object_as_option(&path).await? {
            let data: YakManUserDetails = serde_json::from_str(&content)?;
            return Ok(Some(data));
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
        let data = self
            .get_object_as_option(&path)
            .await?
            .ok_or(AwsS3StorageAdapter::not_found())?;
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
        link: YakManPasswordResetLink,
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

    async fn get_snapshot_lock(&self) -> Result<YakManSnapshotLock, GenericStorageError> {
        todo!()
    }

    async fn save_snapshot_lock(
        &self,
        lock: &YakManSnapshotLock,
    ) -> Result<(), GenericStorageError> {
        todo!()
    }
}

// Helper functions
impl AwsS3StorageAdapter {
    fn get_yakman_dir(&self) -> String {
        let default_dir = String::from(".yakman");
        return self.yakman_dir.as_ref().unwrap_or(&default_dir).to_string();
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
            yakman_dir: None,
            client: client,
            bucket: bucket,
        }
    }
}
