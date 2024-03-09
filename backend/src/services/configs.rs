use std::sync::Arc;

use crate::{
    adapters::{errors::GenericStorageError, KVStorageAdapter},
    error::{CreateConfigError, DeleteConfigError},
    model::YakManConfig,
};

use async_trait::async_trait;
use log::info;

#[async_trait]
pub trait YakManConfigService: Sync + Send {
    async fn get_visible_configs(
        &self,
        project_uuid: Option<String>,
    ) -> Result<Vec<YakManConfig>, GenericStorageError>;

    async fn get_config(
        &self,
        config_name: &str,
    ) -> Result<Option<YakManConfig>, GenericStorageError>;

    async fn create_config(
        &self,
        config_name: &str,
        project_uuid: &str,
    ) -> Result<(), CreateConfigError>;

    async fn delete_config(&self, config_name: &str) -> Result<(), DeleteConfigError>;

    /// Gets all configs including hidden configs
    async fn get_all_configs(
        &self,
        project_uuid: Option<String>,
    ) -> Result<Vec<YakManConfig>, GenericStorageError>;
}

pub struct ConfigService {
    pub adapter: Arc<dyn KVStorageAdapter>,
}

#[async_trait]
impl YakManConfigService for ConfigService {
    async fn get_config(
        &self,
        config_name: &str,
    ) -> Result<Option<YakManConfig>, GenericStorageError> {
        let c = self.adapter.get_configs().await?;
        return Ok(c.into_iter().find(|c| c.name == config_name && !c.hidden));
    }

    async fn get_visible_configs(
        &self,
        project_uuid: Option<String>,
    ) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let configs = self.get_all_configs(project_uuid).await?;
        return Ok(configs.into_iter().filter(|c| !c.hidden).collect());
    }

    async fn create_config(
        &self,
        config_name: &str,
        project_uuid: &str,
    ) -> Result<(), CreateConfigError> {
        let mut configs = self
            .get_all_configs(None)
            .await
            .map_err(|_| CreateConfigError::storage_error("Failed to load configs"))?;

        let mut config = configs.iter_mut().find(|config| config.name == config_name);

        match &mut config {
            Some(&mut ref mut config) => {
                if !config.hidden {
                    return Err(CreateConfigError::duplicate_config(config_name));
                }

                info!("Config '{config_name}' already exists, unhiding it");

                // Config already exists, just unhide it
                config.hidden = false;
                self.adapter.save_configs(configs).await.map_err(|_| {
                    CreateConfigError::storage_error("Failed to update configs file")
                })?;
                return Ok(());
            }
            None => (),
        }

        configs.push(YakManConfig {
            name: String::from(config_name),
            project_uuid: String::from(project_uuid),
            hidden: false,
        });

        // Create instance metadata file
        self.adapter
            .save_instance_metadata(config_name, vec![])
            .await
            .map_err(|_| CreateConfigError::storage_error("Failed to save instance metadata"))?;

        // Create config instances directory
        self.adapter
            .prepare_config_instance_storage(config_name)
            .await
            .map_err(|_| {
                CreateConfigError::storage_error("Failed to create instances directory")
            })?;

        // Create config revisions directory
        self.adapter
            .prepare_revision_instance_storage(config_name)
            .await
            .map_err(|_| {
                CreateConfigError::storage_error("Failed to create revisions directory")
            })?;

        // Add config to base config file
        self.adapter
            .save_configs(configs)
            .await
            .map_err(|_| CreateConfigError::storage_error("Failed to update configs file"))?;

        Ok(())
    }

    async fn delete_config(&self, config_name: &str) -> Result<(), DeleteConfigError> {
        let mut configs = self.get_visible_configs(None).await?;

        if let Some(config) = configs
            .iter_mut()
            .find(|config| config.name == config_name && !config.hidden)
        {
            config.hidden = true;
            self.adapter.save_configs(configs).await?;
            return Ok(());
        }

        return Err(DeleteConfigError::ConfigDoesNotExistError);
    }

    async fn get_all_configs(
        &self,
        project_uuid: Option<String>,
    ) -> Result<Vec<YakManConfig>, GenericStorageError> {
        let configs = match project_uuid {
            Some(project_uuid) => {
                self.adapter
                    .get_configs_by_project_uuid(project_uuid)
                    .await?
            }
            None => self.adapter.get_configs().await?,
        };
        return Ok(configs);
    }
}

impl ConfigService {
    pub fn new(adapter: Arc<dyn KVStorageAdapter>) -> Self {
        Self { adapter: adapter }
    }
}
