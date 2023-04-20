use std::{io::Write, fs::File};

use chrono::Utc;
use rocket::serde::json::serde_json;
use uuid::Uuid;
use yak_man_core::model::{Config, LabelType, Label, ConfigInstance, ConfigInstanceRevision};

use crate::adapters::{errors::{CreateLabelError, ConfigNotFoundError, CreateConfigError}, FileBasedStorageAdapter};

#[async_trait] // TODO: refactor out to other file
pub trait StorageService: Sync + Send {
    async fn get_configs(&self) -> Result<Vec<Config>, ()>;

    // Labels CRUD
    async fn get_labels(&self) -> Result<Vec<LabelType>, ()>;

    async fn create_label(&self, label: LabelType) -> Result<(), CreateLabelError>;

    async fn create_config(&self, config_name: &str) -> Result<(), CreateConfigError>;

    async fn create_config_instance(
        &self,
        config_name: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct FileBasedStorageService {
    pub adapter: Box<dyn FileBasedStorageAdapter>,
}

#[async_trait]
impl StorageService for FileBasedStorageService {
    async fn get_configs(&self) -> Result<Vec<Config>, ()> {
        return Ok(self.adapter.get_configs().await.unwrap());
    }

    async fn get_labels(&self) -> Result<Vec<LabelType>, ()> {
        return Ok(self.adapter.get_labels().await.unwrap());
    }

    async fn create_label(&self, label: LabelType) -> Result<(), CreateLabelError> {
        let mut labels = self.adapter.get_labels().await.unwrap();

        let mut max_prioity: Option<i32> = None;

        // Prevent duplicates
        for lbl in &labels {
            if &lbl.name == &label.name {
                return Err(CreateLabelError::duplicate_label(&label.name));
            }
            if max_prioity.is_none() || lbl.priority > max_prioity.unwrap() {
                max_prioity = Some(lbl.priority);
            }
        }

        if let Some(max_prioity) = max_prioity {
            if max_prioity < label.priority - 1 {
                return Err(CreateLabelError::invalid_priority_error(label.priority));
            }
        }

        for lbl in labels.iter_mut() {
            if lbl.priority >= label.priority {
                lbl.priority += 1;
            }
        }

        labels.push(label);

        self.adapter
            .save_labels(labels)
            .await
            .map_err(|e| CreateLabelError::storage_label(&e.to_string()))?;

        return Ok(());
    }

    async fn create_config_instance(
        &self,
        config_name: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut instances) = self.adapter.get_config_instance_metadata(config_name).await {
            let instance = Uuid::new_v4().to_string();
            let revision_key = Uuid::new_v4().to_string();
            let data_key = Uuid::new_v4().to_string();

            // Create new file with data
            self.adapter.create_config_instance_data_file(config_name, &data_key, data).await?;

            // Create revision
            let revision = ConfigInstanceRevision {
                revision: String::from(&revision_key),
                data_key: String::from(&data_key),
                labels: labels,
                timestamp_ms: Utc::now().timestamp_millis(),
                approved: false,
            };
            self.adapter.update_revision_data(config_name, &revision).await.unwrap();

            // Add new instance to instances and update the instance datafile
            instances.push(ConfigInstance {
                config_name: config_name.to_string(),
                instance: instance,
                labels: revision.labels,
                current_revision: String::from(&revision.revision),
                pending_revision: None,
                revisions: vec![revision.revision],
            });
            self.adapter.update_instance_metadata(config_name, instances)
                .await?;
            println!("Update instance metadata for config: {}", config_name);

            return Ok(());
        }

        return Err(Box::new(ConfigNotFoundError {
            description: format!("Config not found: {config_name}"),
        }));
    }


    async fn create_config(&self, config_name: &str) -> Result<(), CreateConfigError> {
        // let mut configs = self.get_configs().await.unwrap(); // TODO: Handle

        // if configs
        //     .iter()
        //     .find(|config| config.name == config_name)
        //     .is_some()
        // {
        //     return Err(CreateConfigError::duplicate_config(config_name));
        // }

        // configs.push(Config {
        //     name: String::from(config_name),
        //     description: String::from(""), // TODO: support descriptions?
        // });

        // // Create instance metadata file
        // let instace_metadata: Vec<ConfigInstance> = vec![];
        // let data = serde_json::to_string(&InstanceJson {
        //     instances: instace_metadata,
        // })
        // .map_err(|e| CreateConfigError::storage_error("Failed to serialize data to JSON"))?;
        // let yakman_path = self.get_yakman_dir();
        // let path = format!("{yakman_path}/instance-metadata/{config_name}.json");
        // let mut file = File::create(&path)
        //     .map_err(|e| CreateConfigError::storage_error("Failed to instance metadata file"))?;
        // Write::write_all(&mut file, data.as_bytes()).map_err(|e| {
        //     CreateConfigError::storage_error("Failed to update instance metadata file")
        // })?;
        // println!("Created instance metadata file: {}", path);

        // // Create config instances directory
        // let config_instance_dir = self.get_config_instance_dir();
        // let config_instance_path = format!("{config_instance_dir}/{config_name}");
        // fs::create_dir(&config_instance_path).map_err(|e| {
        //     CreateConfigError::storage_error("Failed to create instances directory")
        // })?;
        // println!("Created config instance directory: {config_instance_path}");

        // // Create config revisions directory
        // let revision_instance_dir = self.get_instance_revisions_path();
        // let revision_instance_path = format!("{revision_instance_dir}/{config_name}");
        // fs::create_dir(&revision_instance_path).map_err(|e| {
        //     CreateConfigError::storage_error("Failed to create revisions directory")
        // })?;
        // println!(
        //     "Created config revision directory: {}",
        //     revision_instance_path
        // );

        // // Add config to base config file
        // self.adapter.save_configs(configs);

        Ok(())
    }
}
