use std::{sync::Arc, time::Duration};

use super::{
    id::{generate_config_id, generate_project_id, short_sha},
    password::{hash_password, validate_password},
    StorageService,
};
use crate::{
    adapters::{errors::GenericStorageError, KVStorageAdapter},
    error::{
        ApplyRevisionError, ApproveRevisionError, CreateConfigError, CreateConfigInstanceError,
        CreateLabelError, CreatePasswordResetLinkError, CreateProjectError, CreateTeamError,
        DeleteConfigError, DeleteConfigInstanceError, DeleteProjectError, DeleteTeamError,
        ResetPasswordError, RollbackRevisionError, SaveConfigInstanceError, UpdateProjectError,
        UpdateTeamError,
    },
    model::{
        self,
        request::{CreateTeamPayload, CreateYakManUserPayload, UpdateTeamPayload},
        ConfigDetails, ConfigInstance, ConfigInstanceEvent, ConfigInstanceEventData,
        ConfigInstanceRevision, LabelType, RevisionReviewState, YakManApiKey, YakManConfig,
        YakManLabel, YakManPassword, YakManPasswordResetLink, YakManProject, YakManProjectDetails,
        YakManPublicPasswordResetLink, YakManRole, YakManTeam, YakManTeamDetails, YakManUser,
        YakManUserDetails,
    },
    notifications::{YakManNotificationAdapter, YakManNotificationType},
    services::id::{
        generate_instance_id, generate_revision_id, generate_team_id, generate_user_id,
    },
    settings,
};
use anyhow::bail;
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
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError> {
        return self.adapter.get_projects().await;
    }

    async fn get_project_details(
        &self,
        project_id: &str,
    ) -> Result<Option<YakManProjectDetails>, GenericStorageError> {
        return self.adapter.get_project_details(project_id).await;
    }

    async fn get_config(
        &self,
        config_id: &str,
    ) -> Result<Option<YakManConfig>, GenericStorageError> {
        let c = self.adapter.get_configs().await?;
        return Ok(c.into_iter().find(|c| c.id == config_id && !c.hidden));
    }

    async fn create_project(
        &self,
        project_name: &str,
        notification_settings: Option<model::request::ProjectNotificationSettings>,
    ) -> Result<String, CreateProjectError> {
        let mut projects = self.adapter.get_projects().await?;

        // Prevent duplicates
        for prj in &projects {
            if prj.name == project_name {
                return Err(CreateProjectError::DuplicateNameError {
                    name: String::from(project_name),
                });
            }
        }

        let project_id = generate_project_id();

        let notification_settings = notification_settings.map(|settings| settings.into());

        let project_details: YakManProjectDetails = YakManProjectDetails {
            name: String::from(project_name),
            id: project_id.to_string(),
            notification_settings,
        };

        self.adapter
            .save_project_details(&project_id.to_string(), &project_details)
            .await?;

        projects.push(YakManProject {
            name: String::from(project_name),
            id: project_id.to_string(),
        });

        self.adapter.save_projects(&projects).await?;

        return Ok(project_id.to_string());
    }

    async fn update_project(
        &self,
        project_id: &str,
        project_name: &str,
        notification_settings: Option<model::request::ProjectNotificationSettings>,
    ) -> Result<(), UpdateProjectError> {
        let mut projects = self.adapter.get_projects().await?;

        // Prevent duplicates
        for prj in &projects {
            // Be sure to check that the UUIDs do not match since we should always get at least one match when updating.
            if prj.name == project_name && prj.id != project_id {
                return Err(UpdateProjectError::DuplicateNameError {
                    name: String::from(project_name),
                });
            }
        }

        let Some(mut project_details) = self.adapter.get_project_details(project_id).await? else {
            return Err(UpdateProjectError::ProjectNotFound);
        };
        let Some(project) = projects.iter_mut().find(|p| p.id == project_id) else {
            return Err(UpdateProjectError::ProjectNotFound);
        };

        project.name = project_name.to_string();

        let notification_settings = notification_settings.map(|settings| settings.into());
        project_details.name = project_name.to_string();
        project_details.notification_settings = notification_settings;

        self.adapter
            .save_project_details(project_id, &project_details)
            .await?;
        self.adapter.save_projects(&projects).await?;

        Ok(())
    }

    async fn delete_project(&self, project_id: &str) -> Result<(), DeleteProjectError> {
        let Some(_) = self.adapter.get_project_details(project_id).await? else {
            return Err(DeleteProjectError::ProjectNotFound);
        };
        let mut projects = self.adapter.get_projects().await?;

        let Some(index) = projects.iter().position(|p| p.id == project_id) else {
            return Err(DeleteProjectError::ProjectNotFound);
        };

        projects.remove(index);

        // Delete all of the configs
        let configs = self.adapter.get_configs().await?;

        let project_configs: Vec<_> = configs
            .iter()
            .filter(|p| p.project_id == project_id)
            .collect();

        for config in &project_configs {
            if let Ok(Some(config_details)) = self.adapter.get_config_details(&config.id).await {
                for instance in config_details.instances {
                    let res = self.delete_instance(&config.id, &instance.instance).await;
                    if res.is_err() {
                        log::error!("Failed to delete config {}", config.id);
                    }
                }
            } else {
                log::error!("Failed to delete config {}", config.id);
            }

            let res = self.adapter.delete_config_details(&config.id).await;
            if res.is_err() {
                log::error!("Failed to delete config details {}", config.id);
            }
        }

        let remaining_configs: Vec<_> = configs
            .into_iter()
            .filter(|p| p.project_id != project_id)
            .collect();

        let res = self.adapter.save_configs(&remaining_configs).await;
        if res.is_err() {
            log::error!("Failed to delete configs");
        }
        self.adapter.save_projects(&projects).await?;
        self.adapter.delete_project_details(project_id).await?;

        Ok(())
    }

    async fn get_visible_configs(
        &self,
        project_id: Option<String>,
    ) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let configs = self.get_all_configs(project_id).await?;
        return Ok(configs.into_iter().filter(|c| !c.hidden).collect());
    }

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError> {
        return self.adapter.get_labels().await;
    }

    async fn create_label(&self, mut label: LabelType) -> Result<(), CreateLabelError> {
        let santized_options = label
            .options
            .into_iter()
            .filter_map(|opt| if !opt.is_empty() { Some(opt) } else { None })
            .collect::<Vec<String>>();

        if santized_options.is_empty() {
            return Err(CreateLabelError::EmptyOptionsError);
        }

        label.options = santized_options;

        let mut labels = self.adapter.get_labels().await?;

        // Prevent duplicates
        for lbl in &labels {
            if lbl.name == label.name {
                return Err(CreateLabelError::duplicate_label(&label.name));
            }
        }

        labels.push(label);

        self.adapter.save_labels(&labels).await?;

        return Ok(());
    }

    async fn create_config_instance(
        &self,
        config_id: &str,
        labels: Vec<YakManLabel>,
        data: &str,
        content_type: Option<String>,
        creator_user_id: &str,
    ) -> Result<String, CreateConfigInstanceError> {
        if let Some(mut config_details) = self.adapter.get_config_details(config_id).await? {
            let instances = &mut config_details.instances;
            let instance = generate_instance_id();
            let revision_key: String = generate_revision_id();
            let data_key = Uuid::new_v4().to_string();
            let now = Utc::now().timestamp_millis();

            if !self.validate_labels(&labels).await? {
                return Err(CreateConfigInstanceError::InvalidLabel);
            }

            // Create new file with data
            self.adapter
                .save_instance_data(config_id, &data_key, data)
                .await?;

            // Create revision
            let revision = ConfigInstanceRevision {
                revision: String::from(&revision_key),
                data_key: String::from(&data_key),
                labels: labels,
                timestamp_ms: now,
                review_state: RevisionReviewState::Approved,
                reviewed_by_user_id: Some(creator_user_id.to_string()),
                review_timestamp_ms: Some(now),
                submitted_by_user_id: creator_user_id.to_string(),
                submit_timestamp_ms: now,
                content_type: content_type.unwrap_or(String::from("text/plain")),
            };
            self.adapter.save_revision(config_id, &revision).await?;

            // Add new instance to instances and update the config details
            instances.push(ConfigInstance {
                config_id: config_id.to_string(),
                instance: instance.to_string(),
                labels: revision.labels,
                current_revision: revision.revision.clone(),
                pending_revision: None,
                revisions: vec![revision.revision.clone()],
                changelog: vec![ConfigInstanceEvent {
                    event: ConfigInstanceEventData::Created {
                        new_revision: revision.revision,
                        created_by_user_id: creator_user_id.to_string(),
                    },
                    timestamp_ms: now,
                }],
            });
            self.adapter
                .save_config_details(config_id, &config_details)
                .await?;
            log::info!("Update config details for config: {config_id}");

            if settings::is_notifications_enabled() {
                if let Err(err) = self
                    .send_instance_created_notification(config_id, &instance)
                    .await
                {
                    log::error!("Failed to send notification, {err:?}");
                }
            }

            return Ok(instance);
        }

        return Err(CreateConfigInstanceError::NoConfigFound);
    }

    async fn create_config(
        &self,
        config_name: &str,
        project_id: &str,
    ) -> Result<String, CreateConfigError> {
        let mut configs = self
            .get_all_configs(None)
            .await
            .map_err(|_| CreateConfigError::storage_error("Failed to load configs"))?;

        let mut config = configs.iter_mut().find(|config| config.name == config_name);

        let config_id = generate_config_id();

        // TODO: Review if this logic makes sense
        match &mut config {
            Some(&mut ref mut config) => {
                if !config.hidden {
                    return Err(CreateConfigError::duplicate_config(config_name));
                }

                log::info!("Config '{config_name}' already exists, unhiding it");

                let config_id = config.id.clone();

                // Config already exists, just unhide it
                config.hidden = false;
                self.adapter.save_configs(&configs).await.map_err(|_| {
                    CreateConfigError::storage_error("Failed to update configs file")
                })?;
                return Ok(config_id);
            }
            None => (),
        }

        configs.push(YakManConfig {
            id: config_id.clone(),
            name: String::from(config_name),
            project_id: String::from(project_id),
            hidden: false,
        });

        // Create config details file
        self.adapter
            .save_config_details(
                &config_id,
                &ConfigDetails {
                    config_id: config_id.clone(),
                    config_name: config_name.to_string(),
                    instances: vec![],
                },
            )
            .await
            .map_err(|_| CreateConfigError::storage_error("Failed to save config details"))?;

        // Create config instances directory
        self.adapter
            .prepare_config_instance_storage(&config_id)
            .await
            .map_err(|_| {
                CreateConfigError::storage_error("Failed to create instances directory")
            })?;

        // Create config revisions directory
        self.adapter
            .prepare_revision_instance_storage(&config_id)
            .await
            .map_err(|_| {
                CreateConfigError::storage_error("Failed to create revisions directory")
            })?;

        // Add config to base config file
        self.adapter
            .save_configs(&configs)
            .await
            .map_err(|_| CreateConfigError::storage_error("Failed to update configs file"))?;

        Ok(config_id)
    }

    async fn delete_config(&self, config_id: &str) -> Result<(), DeleteConfigError> {
        let mut configs = self.get_visible_configs(None).await?;

        if let Some(config) = configs
            .iter_mut()
            .find(|config| config.id == config_id && !config.hidden)
        {
            config.hidden = true;
            self.adapter.save_configs(&configs).await?;
            return Ok(());
        }

        return Err(DeleteConfigError::ConfigDoesNotExistError);
    }

    async fn get_config_instance(
        &self,
        config_id: &str,
        instance: &str,
    ) -> Result<Option<ConfigInstance>, GenericStorageError> {
        let config_details = self.adapter.get_config_details(config_id).await?;
        return match config_details {
            Some(config_details) => Ok(config_details
                .instances
                .into_iter()
                .find(|inst| inst.instance == instance)),
            None => Ok(None),
        };
    }

    async fn get_instances_by_config_id(
        &self,
        config_id: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, GenericStorageError> {
        let Some(config_details) = self.adapter.get_config_details(config_id).await? else {
            return Ok(None);
        };
        return Ok(Some(config_details.instances));
    }

    async fn get_config_data(
        &self,
        config_id: &str,
        instance: &str,
    ) -> Result<Option<(String, String)>, GenericStorageError> {
        if let Some(config_details) = self.adapter.get_config_details(config_id).await? {
            let instances = config_details.instances;
            log::info!("Found {} instances", instances.len());

            log::info!("Search for instance ID {}", instance);
            let selected_instance = instances.iter().find(|i| i.instance == instance);

            if let Some(instance) = selected_instance {
                return self
                    .get_data_by_revision(config_id, &instance.current_revision)
                    .await;
            }
            log::info!("No selected instance found");
            return Ok(None);
        }
        return Ok(None);
    }

    async fn submit_new_instance_revision(
        &self,
        config_id: &str,
        instance: &str,
        labels: Vec<YakManLabel>,
        data: &str,
        content_type: Option<String>,
        submitted_by_user_id: &str,
    ) -> Result<String, SaveConfigInstanceError> {
        let mut config_details = self
            .adapter
            .get_config_details(config_id)
            .await?
            .ok_or(SaveConfigInstanceError::InvalidConfig)?;

        let instances = &mut config_details.instances;

        let instance_id = instance;

        let instance = instances
            .iter_mut()
            .find(|inst| inst.instance == instance)
            .ok_or(SaveConfigInstanceError::InvalidInstance)?;

        if !self.validate_labels(&labels).await? {
            return Err(SaveConfigInstanceError::InvalidLabel);
        }

        let revision_key = generate_revision_id();
        let data_key = Uuid::new_v4().to_string();

        // Create new file with data
        self.adapter
            .save_instance_data(config_id, &data_key, data)
            .await?;

        // Create revision
        let now = Utc::now().timestamp_millis();
        let revision = ConfigInstanceRevision {
            revision: String::from(&revision_key),
            data_key: String::from(&data_key),
            labels: labels,
            timestamp_ms: now,
            review_state: RevisionReviewState::Pending,
            reviewed_by_user_id: None,
            review_timestamp_ms: None,
            submitted_by_user_id: submitted_by_user_id.to_string(),
            submit_timestamp_ms: now,
            content_type: content_type.unwrap_or(String::from("text/plain")),
        };
        self.adapter.save_revision(config_id, &revision).await?;

        // Update instance data
        instance.pending_revision = Some(String::from(&revision.revision));
        instance.revisions.push(String::from(&revision.revision));
        instance.changelog.push(ConfigInstanceEvent {
            event: ConfigInstanceEventData::NewRevisionSubmitted {
                previous_revision: instance.current_revision.clone(),
                new_revision: revision.revision.to_string(),
                submitted_by_user_id: submitted_by_user_id.to_string(),
            },
            timestamp_ms: now,
        });

        self.adapter
            .save_config_details(config_id, &config_details)
            .await?;

        log::info!("Updated config details for config: {config_id}");

        if settings::is_notifications_enabled() {
            if let Err(err) = self
                .send_submitted_notification(config_id, instance_id, &revision.revision)
                .await
            {
                log::error!("Failed to send notification, {err:?}");
            }
        }

        return Ok(revision_key);
    }

    async fn get_instance_revisions(
        &self,
        config_id: &str,
        instance: &str,
    ) -> Result<Option<Vec<ConfigInstanceRevision>>, GenericStorageError> {
        let Some(config_details) = self.adapter.get_config_details(config_id).await? else {
            return Ok(None);
        };
        let instances = config_details.instances;

        let instance = match instances.iter().find(|inst| inst.instance == instance) {
            Some(value) => value,
            None => return Ok(None),
        };

        log::info!("found {} revisions", instance.revisions.len());

        let mut revisions: Vec<ConfigInstanceRevision> = vec![];

        for rev in instance.revisions.iter() {
            if let Some(revision) = self.adapter.get_revision(config_id, rev).await? {
                revisions.push(revision);
            }
        }

        return Ok(Some(revisions));
    }

    /// Returns a tuple of (data, content_type)
    async fn get_data_by_revision(
        &self,
        config_id: &str,
        revision: &str,
    ) -> Result<Option<(String, String)>, GenericStorageError> {
        if let Some(revision_data) = self.adapter.get_revision(config_id, revision).await? {
            let key = &revision_data.data_key;
            return Ok(Some((
                self.adapter.get_instance_data(config_id, key).await?,
                revision_data.content_type,
            )));
        }
        info!("Fetching revision not found");
        return Ok(None);
    }

    async fn approve_instance_revision(
        &self,
        config_id: &str,
        instance: &str,
        revision: &str,
        approved_user_id: &str,
    ) -> Result<(), ApproveRevisionError> {
        let Some(mut config_details) = self.adapter.get_config_details(config_id).await? else {
            return Err(ApproveRevisionError::InvalidConfig);
        };
        let instances = &mut config_details.instances;

        let instance_id = instance;
        let instance = match instances.iter_mut().find(|i| i.instance == instance) {
            Some(instance) => instance,
            None => return Err(ApproveRevisionError::InvalidInstance),
        };

        // Verify instance is the pending revision
        if let Some(pending_revision) = &instance.pending_revision {
            if pending_revision != revision {
                return Err(ApproveRevisionError::InvalidRevision);
            }
        } else {
            return Err(ApproveRevisionError::InvalidRevision);
        }

        let mut revision_data = match self.adapter.get_revision(config_id, revision).await.ok() {
            Some(Some(revision_data)) => revision_data,
            None | Some(None) => return Err(ApproveRevisionError::InvalidRevision),
        };

        let now = Utc::now().timestamp_millis();
        revision_data.review_state = RevisionReviewState::Approved;
        revision_data.reviewed_by_user_id = Some(approved_user_id.to_string());
        revision_data.review_timestamp_ms = Some(now);
        self.adapter
            .save_revision(config_id, &revision_data)
            .await?;

        if !instance.revisions.contains(&String::from(revision)) {
            instance.revisions.push(String::from(revision));
        }
        instance.changelog.push(ConfigInstanceEvent {
            event: ConfigInstanceEventData::NewRevisionApproved {
                new_revision: revision.to_string(),
                approver_by_user_id: approved_user_id.to_string(),
            },
            timestamp_ms: now,
        });

        self.adapter
            .save_config_details(config_id, &config_details)
            .await?;

        if settings::is_notifications_enabled() {
            if let Err(err) = self
                .send_approved_notification(config_id, instance_id, &revision_data.revision)
                .await
            {
                log::error!("Failed to send notification, {err:?}");
            }
        }

        return Ok(());
    }

    async fn apply_instance_revision(
        &self,
        config_id: &str,
        instance: &str,
        revision: &str,
        applied_by_user_id: &str,
    ) -> Result<(), ApplyRevisionError> {
        let Some(mut config_details) = self.adapter.get_config_details(config_id).await? else {
            return Err(ApplyRevisionError::InvalidConfig);
        };
        let instances = &mut config_details.instances;

        let instance_id = instance;
        let instance = match instances.iter_mut().find(|i| i.instance == instance) {
            Some(instance) => instance,
            None => return Err(ApplyRevisionError::InvalidInstance),
        };

        // Verify instance is the pending revision
        if let Some(pending_revision) = &instance.pending_revision {
            if pending_revision != revision {
                return Err(ApplyRevisionError::InvalidRevision);
            }
        } else {
            return Err(ApplyRevisionError::InvalidRevision);
        }

        let revision_data = match self.adapter.get_revision(config_id, revision).await.ok() {
            Some(Some(revision_data)) => revision_data,
            None | Some(None) => return Err(ApplyRevisionError::InvalidRevision),
        };

        if revision_data.review_state != RevisionReviewState::Approved {
            return Err(ApplyRevisionError::NotApproved);
        }

        let now = Utc::now().timestamp_millis();
        instance.changelog.push(ConfigInstanceEvent {
            event: ConfigInstanceEventData::Updated {
                previous_revision: instance.current_revision.clone(),
                new_revision: String::from(revision),
                applied_by_user_id: String::from(applied_by_user_id),
            },
            timestamp_ms: now,
        });
        instance.current_revision = String::from(revision);
        instance.pending_revision = None;
        instance.labels = revision_data.labels;

        if !instance.revisions.contains(&String::from(revision)) {
            instance.revisions.push(String::from(revision));
        }

        self.adapter
            .save_config_details(config_id, &config_details)
            .await?;

        if settings::is_notifications_enabled() {
            if let Err(err) = self
                .send_applied_notification(config_id, instance_id, revision)
                .await
            {
                log::error!("Failed to send notification, {err:?}");
            }
        }

        return Ok(());
    }

    async fn reject_instance_revision(
        &self,
        config_id: &str,
        instance: &str,
        revision: &str,
        rejected_by_user_id: &str,
    ) -> Result<(), ApplyRevisionError> {
        let Some(mut config_details) = self.adapter.get_config_details(config_id).await? else {
            return Err(ApplyRevisionError::InvalidConfig);
        };
        let instances = &mut config_details.instances;

        let instance_id = instance;
        let instance = match instances.iter_mut().find(|i| i.instance == instance) {
            Some(instance) => instance,
            None => return Err(ApplyRevisionError::InvalidInstance),
        };

        let mut revision_data = match self.adapter.get_revision(config_id, revision).await.ok() {
            Some(Some(revision_data)) => revision_data,
            None | Some(None) => return Err(ApplyRevisionError::InvalidRevision),
        };

        let now = Utc::now().timestamp_millis();
        revision_data.review_state = RevisionReviewState::Rejected;
        revision_data.reviewed_by_user_id = Some(rejected_by_user_id.to_string());
        revision_data.review_timestamp_ms = Some(now);

        instance.pending_revision = None;

        if let Some(index) = instance.revisions.iter().position(|x| *x == revision) {
            instance.revisions.remove(index);
        }

        instance.changelog.push(ConfigInstanceEvent {
            event: ConfigInstanceEventData::NewRevisionRejected {
                new_revision: revision.to_string(),
                rejected_by_user_id: rejected_by_user_id.to_string(),
            },
            timestamp_ms: now,
        });

        self.adapter
            .save_revision(config_id, &revision_data)
            .await?;

        self.adapter
            .save_config_details(config_id, &config_details)
            .await?;

        if settings::is_notifications_enabled() {
            if let Err(err) = self
                .send_reject_notification(config_id, instance_id, revision)
                .await
            {
                log::error!("Failed to send notification, {err:?}");
            }
        }

        return Ok(());
    }

    async fn rollback_instance_revision(
        &self,
        config_id: &str,
        instance: &str,
        revision: &str,
        rollback_by_user_id: &str,
    ) -> Result<String, RollbackRevisionError> {
        let mut config_details = self
            .adapter
            .get_config_details(config_id)
            .await?
            .ok_or(RollbackRevisionError::InvalidConfig)?;

        let instances = &mut config_details.instances;

        let instance = instances
            .iter_mut()
            .find(|inst| inst.instance == instance)
            .ok_or(RollbackRevisionError::InvalidInstance)?;

        let previous_revision = self
            .adapter
            .get_revision(config_id, revision)
            .await?
            .ok_or(RollbackRevisionError::InvalidRevision)?;

        let revision_key = generate_revision_id();

        // Create revision
        let now = Utc::now().timestamp_millis();
        let revision = ConfigInstanceRevision {
            revision: String::from(&revision_key),
            data_key: previous_revision.data_key,
            labels: previous_revision.labels.clone(),
            timestamp_ms: now,
            review_state: RevisionReviewState::Pending,
            reviewed_by_user_id: None,
            review_timestamp_ms: Some(now),
            submitted_by_user_id: rollback_by_user_id.to_string(),
            submit_timestamp_ms: now,
            content_type: previous_revision.content_type,
        };
        self.adapter.save_revision(config_id, &revision).await?;

        // Update instance data
        instance.pending_revision = Some(String::from(&revision.revision));
        instance.revisions.push(String::from(&revision.revision));

        self.adapter
            .save_config_details(config_id, &config_details)
            .await?;
        log::info!("Updated config details for config: {config_id}");
        return Ok(revision_key);
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
                id: generate_user_id(),
            };

            let admin_user_details = YakManUserDetails {
                user_id: admin_user.id.clone(),
                global_roles: vec![YakManRole::Admin],
                roles: vec![],
                profile_picture: None,
                team_ids: vec![],
            };

            self.adapter
                .save_user_details(&admin_user.id, &admin_user_details)
                .await?;

            self.adapter.save_users(&vec![admin_user]).await?;
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
                                &YakManPassword {
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

    async fn get_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<YakManUser>, GenericStorageError> {
        return self.adapter.get_user_by_email(email).await;
    }

    async fn get_user_by_id(
        &self,
        user_id: &str,
    ) -> Result<Option<YakManUser>, GenericStorageError> {
        return self.adapter.get_user_by_id(user_id).await;
    }

    async fn get_user_details(
        &self,
        user_id: &str,
    ) -> Result<Option<YakManUserDetails>, GenericStorageError> {
        return self.adapter.get_user_details(user_id).await;
    }

    async fn save_user_details(
        &self,
        user_id: &str,
        details: YakManUserDetails,
    ) -> Result<(), GenericStorageError> {
        return self.adapter.save_user_details(user_id, &details).await;
    }

    async fn create_user(
        &self,
        payload: CreateYakManUserPayload,
    ) -> Result<String, GenericStorageError> {
        let mut users = self.get_users().await?;

        // TODO: Prevent duplicate emails

        let user_id = generate_user_id();

        users.push(YakManUser {
            email: payload.email.clone(),
            id: user_id.clone(),
            role: payload.role.clone(),
        });

        // TODO: Handle roles properly
        let user_details = YakManUserDetails {
            user_id: user_id.clone(),
            profile_picture: None,
            global_roles: vec![],
            roles: vec![],
            team_ids: vec![],
        };

        self.adapter
            .save_user_details(&user_id, &user_details)
            .await?;

        self.adapter.save_users(&users).await?;

        Ok(user_id)
    }

    async fn get_teams(&self) -> Result<Vec<YakManTeam>, GenericStorageError> {
        return self.adapter.get_teams().await;
    }

    async fn get_team_details(
        &self,
        team_id: &str,
    ) -> Result<Option<YakManTeamDetails>, GenericStorageError> {
        return self.adapter.get_team_details(team_id).await;
    }

    async fn create_team(&self, payload: CreateTeamPayload) -> Result<String, CreateTeamError> {
        let team_name = payload.name;

        let mut teams = self.adapter.get_teams().await?;
        if teams.iter().any(|t| t.name == team_name) {
            return Err(CreateTeamError::DuplicateTeam);
        }

        let mut user_details: Vec<YakManUserDetails> = vec![];

        for user_id in &payload.team_member_user_ids {
            let Ok(Some(details)) = self.adapter.get_user_details(user_id).await else {
                log::error!("Failed to get user details for user ID {user_id}");
                continue;
            };
            user_details.push(details);
        }

        let team_id = generate_team_id();

        teams.push(YakManTeam {
            id: team_id.clone(),
            name: team_name.clone(),
        });

        self.adapter
            .save_team_details(
                &team_id.clone(),
                &YakManTeamDetails {
                    id: team_id.clone(),
                    name: team_name,
                    roles: payload.roles,
                    global_roles: payload.global_roles,
                    member_user_ids: payload.team_member_user_ids,
                },
            )
            .await?;

        self.adapter.save_teams(&teams).await?;

        for mut user in user_details {
            user.team_ids.push(team_id.clone());
            let user_id = user.user_id.clone();

            let res = self.adapter.save_user_details(&user_id, &user).await;
            if res.is_err() {
                log::error!("Failed to save user id: {user_id}");
            }
        }

        return Ok(team_id);
    }

    async fn update_team(
        &self,
        team_id: &str,
        payload: UpdateTeamPayload,
    ) -> Result<(), UpdateTeamError> {
        let team_name = &payload.name;

        let mut teams = self.adapter.get_teams().await?;
        if teams
            .iter()
            .any(|t| t.name == *team_name && t.id != team_id)
        {
            return Err(UpdateTeamError::DuplicateTeam);
        }

        let Some(team) = teams.iter_mut().find(|t| t.id == team_id) else {
            return Err(UpdateTeamError::TeamNotFound);
        };
        team.name = team_name.clone();

        let Some(mut team_details) = self.adapter.get_team_details(team_id).await? else {
            return Err(UpdateTeamError::TeamNotFound);
        };

        let mut user_details: Vec<YakManUserDetails> = vec![];

        for user_id in &payload.team_member_user_ids {
            let Ok(Some(details)) = self.adapter.get_user_details(user_id).await else {
                log::error!("Failed to get user details for user ID {user_id}");
                continue;
            };
            user_details.push(details);
        }

        let mut user_details_to_delete: Vec<YakManUserDetails> = vec![];
        let mut user_ids_to_delete = team_details.member_user_ids.clone();
        user_ids_to_delete.retain(|uid| !payload.team_member_user_ids.contains(uid));
        for user_id in &user_ids_to_delete {
            let Ok(Some(details)) = self.adapter.get_user_details(user_id).await else {
                log::error!("Failed to get user details for user ID {user_id}");
                continue;
            };
            user_details_to_delete.push(details);
        }

        team_details.name = team_name.clone();
        team_details.global_roles = payload.global_roles;
        team_details.roles = payload.roles;
        team_details.member_user_ids = payload.team_member_user_ids;

        for mut user in user_details_to_delete {
            if let Some(team_id_index) = user.team_ids.iter().position(|tid| tid == team_id) {
                user.team_ids.remove(team_id_index);
                let user_id = user.user_id.clone();
                let res = self.adapter.save_user_details(&user_id, &user).await;
                if res.is_err() {
                    log::error!("Failed to save user id: {user_id} (remove from team)");
                }
            }
        }

        self.adapter
            .save_team_details(team_id, &team_details)
            .await?;

        self.adapter.save_teams(&teams).await?;

        for mut user in user_details {
            if !user.team_ids.iter().any(|tid| tid == team_id) {
                user.team_ids.push(team_id.to_string());
                let user_id = user.user_id.clone();
                let res = self.adapter.save_user_details(&user_id, &user).await;
                if res.is_err() {
                    log::error!("Failed to save user id: {user_id}");
                }
            }
        }

        return Ok(());
    }

    async fn delete_team(&self, team_id: &str) -> Result<(), DeleteTeamError> {
        let mut teams = self.adapter.get_teams().await?;
        let Some(team_details) = self.adapter.get_team_details(team_id).await? else {
            return Err(DeleteTeamError::TeamNotFound);
        };

        teams.retain(|team| team.id != team_id);

        let mut users = vec![];
        for user_id in &team_details.member_user_ids {
            if let Some(user_details) = self.adapter.get_user_details(user_id).await? {
                users.push(user_details);
            } else {
                log::error!("Failed to get user with id {user_id}, skipping removing team");
            }
        }

        for mut user in users {
            user.team_ids.retain(|id| id != team_id);
            let user_id = user.user_id.clone();
            let res = self.adapter.save_user_details(&user_id, &user).await;
            if res.is_err() {
                log::error!("Failed to save user with id {user_id}, skipping removing team");
            }
        }

        self.adapter.delete_team_details(team_id).await?;
        self.adapter.save_teams(&teams).await?;

        return Ok(());
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

        return self.adapter.save_api_keys(&api_keys).await;
    }

    async fn delete_api_key(&self, id: &str) -> Result<(), GenericStorageError> {
        let mut api_keys = self.get_api_keys().await?;

        if let Some(index) = api_keys.iter().position(|k| k.id == id) {
            api_keys.remove(index);
        }

        self.put_api_keys_cache(&api_keys);
        return self.adapter.save_api_keys(&api_keys).await;
    }

    async fn delete_instance(
        &self,
        config_id: &str,
        instance: &str,
    ) -> Result<(), DeleteConfigInstanceError> {
        let mut config_details = self
            .adapter
            .get_config_details(config_id)
            .await?
            .ok_or(DeleteConfigInstanceError::InvalidConfig)?;

        let config_instance = config_details
            .instances
            .iter()
            .find(|i| i.instance == instance)
            .ok_or(DeleteConfigInstanceError::InvalidInstance)?
            .clone();

        let remaining_instances: Vec<_> = config_details
            .instances
            .into_iter()
            .filter(|i| i.instance != instance)
            .collect();

        config_details.instances = remaining_instances;

        self.adapter
            .save_config_details(config_id, &config_details)
            .await?;

        for revision in config_instance.revisions {
            if let Err(e) = self.adapter.delete_revision(config_id, &revision).await {
                log::error!("Failed to delete revision ({revision}) {e:?}");
            }
        }

        return Ok(());
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
        user_id: &str,
    ) -> Result<YakManPublicPasswordResetLink, CreatePasswordResetLinkError> {
        let user = match self.get_user_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(CreatePasswordResetLinkError::InvalidUser),
        };

        let id = short_sha(&Uuid::new_v4().to_string());
        let id_hash = sha256::digest(&id);

        let email = user.email;
        let email_hash = sha256::digest(&email);

        let expiration =
            Utc::now() + chrono::Duration::try_days(2).expect("2 days will not be out of bounds");

        let password_reset_link = YakManPasswordResetLink {
            email_hash,
            expiration_timestamp_ms: expiration.timestamp_millis(),
        };

        self.adapter
            .save_password_reset_link(&id_hash, &password_reset_link)
            .await?;

        return Ok(YakManPublicPasswordResetLink {
            id,
            user_id: user_id.to_string(),
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

        // Validate user_id match email hash from storage
        let user = match self.get_user_by_id(&reset_link.user_id).await? {
            Some(user) => user,
            None => return Err(ResetPasswordError::InvalidUser),
        };
        let email_hash = sha256::digest(&user.email);
        if email_hash != password_reset_link.email_hash {
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
                &YakManPassword {
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
        user_id: &str,
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

        // Validate user_id match email hash from storage
        let Some(user) = self.get_user_by_id(user_id).await? else {
            return Ok(false);
        };

        let email_hash = sha256::digest(&user.email);
        return Ok(email_hash == password_reset_link.email_hash);
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

    async fn send_instance_created_notification(
        &self,
        config_id: &str,
        instance: &str,
    ) -> anyhow::Result<()> {
        let (project, config) = self.get_data_to_send_notification(config_id).await?;

        let Some(notification_settings) = project.notification_settings else {
            return Ok(()); // No notification settings configured for project
        };

        if !notification_settings.events.is_instance_created_enabled {
            return Ok(()); // Project does not have this notification enabled
        }

        let notification_adapter: Arc<dyn YakManNotificationAdapter + Send + Sync> =
            notification_settings.settings.into();
        notification_adapter
            .send_notification(YakManNotificationType::InstanceCreated {
                project_name: project.name.to_string(),
                config_name: config.name,
                instance: instance.to_string(),
            })
            .await?;

        return Ok(());
    }

    async fn send_submitted_notification(
        &self,
        config_id: &str,
        instance: &str,
        revision: &str,
    ) -> anyhow::Result<()> {
        let (project, config) = self.get_data_to_send_notification(config_id).await?;

        let Some(notification_settings) = project.notification_settings else {
            return Ok(()); // No notification settings configured for project
        };

        if !notification_settings.events.is_revision_submitted_enabled {
            return Ok(()); // Project does not have this notification enabled
        }

        let notification_adapter: Arc<dyn YakManNotificationAdapter + Send + Sync> =
            notification_settings.settings.into();
        notification_adapter
            .send_notification(YakManNotificationType::RevisionReviewSubmitted {
                project_name: project.name.to_string(),
                config_name: config.name,
                instance: instance.to_string(),
                revision: revision.to_string(),
            })
            .await?;

        return Ok(());
    }

    async fn send_approved_notification(
        &self,
        config_id: &str,
        instance: &str,
        revision: &str,
    ) -> anyhow::Result<()> {
        let (project, config) = self.get_data_to_send_notification(config_id).await?;

        let Some(notification_settings) = project.notification_settings else {
            return Ok(()); // No notification settings configured for project
        };

        if !notification_settings.events.is_revision_approved_enabled {
            return Ok(()); // Project does not have this notification enabled
        }

        let notification_adapter: Arc<dyn YakManNotificationAdapter + Send + Sync> =
            notification_settings.settings.into();
        notification_adapter
            .send_notification(YakManNotificationType::RevisionReviewApproved {
                project_name: project.name.to_string(),
                config_name: config.name,
                instance: instance.to_string(),
                revision: revision.to_string(),
            })
            .await?;

        return Ok(());
    }

    async fn send_applied_notification(
        &self,
        config_id: &str,
        instance: &str,
        revision: &str,
    ) -> anyhow::Result<()> {
        let (project, config) = self.get_data_to_send_notification(config_id).await?;

        let Some(notification_settings) = project.notification_settings else {
            return Ok(()); // No notification settings configured for project
        };

        if !notification_settings.events.is_instance_updated_enabled {
            return Ok(()); // Project does not have this notification enabled
        }

        let notification_adapter: Arc<dyn YakManNotificationAdapter + Send + Sync> =
            notification_settings.settings.into();
        notification_adapter
            .send_notification(YakManNotificationType::RevisionReviewApplied {
                project_name: project.name.to_string(),
                config_name: config.name,
                instance: instance.to_string(),
                revision: revision.to_string(),
            })
            .await?;

        return Ok(());
    }

    async fn send_reject_notification(
        &self,
        config_id: &str,
        instance: &str,
        revision: &str,
    ) -> anyhow::Result<()> {
        let (project, config) = self.get_data_to_send_notification(config_id).await?;

        let Some(notification_settings) = project.notification_settings else {
            return Ok(()); // No notification settings configured for project
        };

        if !notification_settings.events.is_revision_reject_enabled {
            return Ok(()); // Project does not have this notification enabled
        }

        let notification_adapter: Arc<dyn YakManNotificationAdapter + Send + Sync> =
            notification_settings.settings.into();
        notification_adapter
            .send_notification(YakManNotificationType::RevisionReviewRejected {
                project_name: project.name.to_string(),
                config_name: config.name.to_string(),
                instance: instance.to_string(),
                revision: revision.to_string(),
            })
            .await?;

        return Ok(());
    }

    async fn get_data_to_send_notification(
        &self,
        config_id: &str,
    ) -> anyhow::Result<(YakManProjectDetails, YakManConfig)> {
        let configs = self.adapter.get_configs().await?;
        let Some(config) = configs.into_iter().find(|c| c.id == config_id) else {
            bail!("Could not find config {config_id}")
        };

        let Some(project) = self.adapter.get_project_details(&config.project_id).await? else {
            bail!("Could not find project {}", config.project_id)
        };

        return Ok((project, config));
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

    /// Gets all configs including hidden configs
    async fn get_all_configs(
        &self,
        project_id: Option<String>,
    ) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let configs = match project_id {
            Some(project_id) => self.adapter.get_configs_by_project_id(&project_id).await?,
            None => self.adapter.get_configs().await?,
        };
        return Ok(configs);
    }

    /// Returns true if all labels exist and have valid values
    async fn validate_labels(
        &self,
        labels: &Vec<YakManLabel>,
    ) -> Result<bool, GenericStorageError> {
        let all_labels = self.get_labels().await?;
        for label in labels {
            if let Some(label_type) = all_labels.iter().find(|l| l.name == label.label_type) {
                if !label_type.options.iter().any(|opt| opt == &label.value) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }
        return Ok(true);
    }
}
