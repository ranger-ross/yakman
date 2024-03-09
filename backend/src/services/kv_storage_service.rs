use std::{sync::Arc, time::Duration};

use super::{
    password::{hash_password, validate_password},
    StorageService,
};
use crate::{
    adapters::{errors::GenericStorageError, KVStorageAdapter},
    error::{CreateLabelError, CreatePasswordResetLinkError, ResetPasswordError},
    model::{
        LabelType, YakManApiKey, YakManPassword, YakManPasswordResetLink,
        YakManPublicPasswordResetLink, YakManRole, YakManUser, YakManUserDetails,
    },
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use async_trait::async_trait;
use chrono::Utc;
use log::info;
use moka::sync::{Cache, CacheBuilder};
use uuid::Uuid;

pub struct KVStorageService {
    pub adapter: Arc<dyn KVStorageAdapter>,
    /// The cache key is the ID as a string
    pub api_key_id_cache: Cache<String, YakManApiKey>,
    /// The cache key is the token hash as a string
    pub api_key_hash_cache: Cache<String, YakManApiKey>,
}

#[async_trait]
impl StorageService for KVStorageService {
    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError> {
        return Ok(self.adapter.get_labels().await?);
    }

    async fn create_label(&self, mut label: LabelType) -> Result<(), CreateLabelError> {
        let santized_options = label
            .options
            .into_iter()
            .filter_map(|opt| if !opt.is_empty() { Some(opt) } else { None })
            .collect::<Vec<String>>();

        if santized_options.len() == 0 {
            return Err(CreateLabelError::EmptyOptionsError);
        }

        label.options = santized_options;

        let mut labels = self.adapter.get_labels().await?;

        // Prevent duplicates
        for lbl in &labels {
            if &lbl.name == &label.name {
                return Err(CreateLabelError::duplicate_label(&label.name));
            }
        }

        labels.push(label);

        self.adapter.save_labels(labels).await?;

        return Ok(());
    }

    async fn get_config_data(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<(String, String)>, GenericStorageError> {
        if let Some(instances) = self.adapter.get_instance_metadata(config_name).await? {
            info!("Found {} instances", instances.len());

            info!("Search for instance ID {}", instance);
            let selected_instance = instances.iter().find(|i| i.instance == instance);

            if let Some(instance) = selected_instance {
                return self
                    .get_data_by_revision(config_name, &instance.current_revision)
                    .await;
            }
            info!("No selected instance found");
            return Ok(None);
        }
        return Ok(None);
    }

    /// Returns a tuple of (data, content_type)
    async fn get_data_by_revision(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<Option<(String, String)>, GenericStorageError> {
        if let Some(revision_data) = self.adapter.get_revision(config_name, revision).await? {
            let key = &revision_data.data_key;
            return Ok(Some((
                self.adapter.get_instance_data(config_name, key).await?,
                revision_data.content_type,
            )));
        }
        info!("Fetching revision not found");
        return Ok(None);
    }

    async fn initialize_storage(&self) -> Result<(), GenericStorageError> {
        log::info!("initializing local storage adapter");
        let now = Utc::now().timestamp_millis();

        self.adapter.initialize_yakman_storage().await?;

        let users = self.adapter.get_users().await?;

        // During first time launch, add the default email as a global admin
        if users.is_empty() {
            let admin_user = YakManUser {
                email: std::env::var("YAKMAN_DEFAULT_ADMIN_USER_EMAIL")
                    .expect("No users found and 'YAKMAN_DEFAULT_ADMIN_USER_EMAIL' is not set"),
                role: Some(YakManRole::Admin),
                uuid: Uuid::new_v4().to_string(),
            };

            let admin_user_details = YakManUserDetails {
                global_roles: vec![YakManRole::Admin],
                roles: vec![],
                profile_picture: None,
            };

            self.adapter
                .save_user_details(&admin_user.uuid, admin_user_details)
                .await?;

            self.adapter.save_users(vec![admin_user]).await?;
        }

        // Set the default admin password
        if let Ok(email) = std::env::var("YAKMAN_DEFAULT_ADMIN_USER_EMAIL") {
            if let Ok(default_password) = std::env::var("YAKMAN_DEFAULT_ADMIN_USER_PASSWORD") {
                let email_hash = sha256::digest(&email);

                // Don't set the password if it already exists
                match self.adapter.get_password(&email_hash).await {
                    Ok(None) => {
                        log::info!("Saving default admin password");
                        // Example from: https://docs.rs/argon2/latest/argon2
                        let salt = SaltString::generate(&mut OsRng);
                        let argon2 = Argon2::default();
                        let password_hash = argon2
                            .hash_password(default_password.as_bytes(), &salt)
                            .map_err(|e| {
                                GenericStorageError::new(
                                    "Failed to hash default password".to_string(),
                                    e.to_string(),
                                )
                            })?
                            .to_string();

                        self.adapter
                            .save_password(
                                &email_hash,
                                YakManPassword {
                                    hash: password_hash,
                                    timestamp: now,
                                },
                            )
                            .await?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

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

    async fn get_api_keys(&self) -> Result<Vec<YakManApiKey>, GenericStorageError> {
        let api_keys = self.adapter.get_api_keys().await?;
        self.put_api_keys_cache(&api_keys);
        return Ok(api_keys);
    }

    async fn get_api_key_by_id(
        &self,
        id: &str,
    ) -> Result<Option<YakManApiKey>, GenericStorageError> {
        if let Some(key) = self.api_key_id_cache.get(id) {
            return Ok(Some(key));
        }

        let api_keys: Vec<YakManApiKey> = self.get_api_keys().await?;
        return Ok(api_keys.into_iter().find(|key| key.id == id));
    }

    async fn get_api_key_by_hash(
        &self,
        hash: &str,
    ) -> Result<Option<YakManApiKey>, GenericStorageError> {
        if let Some(key) = self.api_key_hash_cache.get(hash) {
            return Ok(Some(key));
        }

        let api_keys: Vec<YakManApiKey> = self.get_api_keys().await?;
        return Ok(api_keys.into_iter().find(|key| key.hash == hash));
    }

    async fn save_api_key(&self, api_key: YakManApiKey) -> Result<(), GenericStorageError> {
        let mut api_keys = self.get_api_keys().await?;

        if let Some(index) = api_keys.iter().position(|k| k.id == api_key.id) {
            api_keys[index] = api_key;
        } else {
            api_keys.push(api_key);
        }

        self.put_api_keys_cache(&api_keys);

        return self.adapter.save_api_keys(api_keys).await;
    }

    async fn delete_api_key(&self, id: &str) -> Result<(), GenericStorageError> {
        let mut api_keys = self.get_api_keys().await?;

        if let Some(index) = api_keys.iter().position(|k| k.id == id) {
            api_keys.remove(index);
        }

        self.put_api_keys_cache(&api_keys);
        return self.adapter.save_api_keys(api_keys).await;
    }

    async fn get_password_by_email(
        &self,
        email: &str,
    ) -> Result<Option<YakManPassword>, GenericStorageError> {
        let email_hash = sha256::digest(email);
        return self.adapter.get_password(&email_hash).await;
    }

    async fn create_password_reset_link(
        &self,
        user_uuid: &str,
    ) -> Result<YakManPublicPasswordResetLink, CreatePasswordResetLinkError> {
        let user = match self.get_user_by_uuid(user_uuid).await? {
            Some(user) => user,
            None => return Err(CreatePasswordResetLinkError::InvalidUser),
        };

        let id = short_sha(&Uuid::new_v4().to_string());
        let id_hash = sha256::digest(&id);

        let email = user.email;
        let email_hash = sha256::digest(&email);

        let expiration = Utc::now() + chrono::Duration::days(2);

        let password_reset_link = YakManPasswordResetLink {
            email_hash,
            expiration_timestamp_ms: expiration.timestamp_millis(),
        };

        self.adapter
            .save_password_reset_link(&id_hash, password_reset_link)
            .await?;

        return Ok(YakManPublicPasswordResetLink {
            id,
            user_uuid: user_uuid.to_string(),
        });
    }

    async fn reset_password_with_link(
        &self,
        reset_link: YakManPublicPasswordResetLink,
        password: &str,
    ) -> Result<(), ResetPasswordError> {
        let now = Utc::now().timestamp_millis();

        let id = sha256::digest(&reset_link.id);
        let password_reset_link = match self.adapter.get_password_reset_link(&id).await? {
            Some(password_reset_link) => password_reset_link,
            None => {
                return Err(ResetPasswordError::ResetLinkNotFound);
            }
        };

        // Validate user_uuid match email hash from storage
        let user = match self.get_user_by_uuid(&reset_link.user_uuid).await? {
            Some(user) => user,
            None => return Err(ResetPasswordError::InvalidUser),
        };
        let email_hash = sha256::digest(&user.email);
        if &email_hash != &password_reset_link.email_hash {
            return Err(ResetPasswordError::InvalidEmail);
        }

        // Validate expiration
        if password_reset_link.expiration_timestamp_ms < now {
            return Err(ResetPasswordError::ResetLinkExpired);
        }

        if let Err(err) = validate_password(password) {
            return Err(ResetPasswordError::PasswordValidationError { error: err });
        }

        let password_hash = hash_password(password)
            .map_err(|err| ResetPasswordError::PasswordHashError { error: err })?;
        self.adapter
            .save_password(
                &email_hash,
                YakManPassword {
                    hash: password_hash,
                    timestamp: now,
                },
            )
            .await?;

        self.adapter.delete_password_reset_link(&id).await?;

        Ok(())
    }

    async fn validate_password_reset_link(
        &self,
        id: &str,
        user_uuid: &str,
    ) -> Result<bool, GenericStorageError> {
        let id = sha256::digest(id);
        let password_reset_link = match self.adapter.get_password_reset_link(&id).await? {
            Some(password_reset_link) => password_reset_link,
            None => return Ok(false),
        };

        let now = Utc::now().timestamp_millis();

        // Validate expiration
        if password_reset_link.expiration_timestamp_ms < now {
            return Ok(false);
        }

        // Validate user_uuid match email hash from storage
        let user = match self.get_user_by_uuid(user_uuid).await? {
            Some(user) => user,
            None => return Ok(false),
        };

        let email_hash = sha256::digest(&user.email);
        return Ok(&email_hash == &password_reset_link.email_hash);
    }
}

impl KVStorageService {
    fn put_api_keys_cache(&self, api_keys: &Vec<YakManApiKey>) {
        // Update caches
        for key in api_keys {
            self.api_key_id_cache
                .insert(key.id.to_string(), key.clone());
            self.api_key_hash_cache
                .insert(key.hash.to_string(), key.clone());
        }
    }

    pub fn new(adapter: Arc<dyn KVStorageAdapter>) -> KVStorageService {
        let api_key_id_cache = CacheBuilder::new(10_000)
            .time_to_live(Duration::from_secs(60))
            .build();
        let api_key_hash_cache = CacheBuilder::new(10_000)
            .time_to_live(Duration::from_secs(60))
            .build();

        KVStorageService {
            adapter: adapter,
            api_key_id_cache,
            api_key_hash_cache,
        }
    }
}

/// Returns a 12 character string representation of a SHA256
fn short_sha(input: &str) -> String {
    let sha: String = sha256::digest(input);
    return sha[0..12].to_string();
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_short_sha() {
        let result = short_sha("hello world");
        assert_eq!(result, "b94d27b9934d");

        let result = short_sha("foo");
        assert_eq!(result, "2c26b46b68ff");

        let result = short_sha("bar");
        assert_eq!(result, "fcde2b2edba5");

        let result = short_sha("ade10004-41df-4bf6-88b9-d768afab674f");
        assert_eq!(result, "8146205a8d27");
    }
}
