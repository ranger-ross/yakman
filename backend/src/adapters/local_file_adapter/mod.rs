use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use chrono::Utc;
use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};

use uuid::Uuid;
use yak_man_core::model::{Config, ConfigInstance, ConfigInstanceRevision, Label, LabelType};

use crate::adapters::utils::select_instance;

use super::{
    errors::ApproveRevisionError, CreateConfigError, FileBasedStorageAdapter, GenericStorageError,
};

pub struct LocalFileStorageAdapter {
    pub path: String,
}

pub fn create_local_file_adapter() -> LocalFileStorageAdapter {
    return LocalFileStorageAdapter {
        path: "/home/ross/projects/config-manager/testing-directory".to_string(),
    };
}

const YAK_MAN_DIR: &str = ".yakman"; // TODO: clean up
const DATA_DIR: &str = "config-instances"; // TODO: clean up

#[derive(Debug, Serialize, Deserialize)]
struct LabelJson {
    labels: Vec<LabelType>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigJson {
    configs: Vec<Config>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstanceJson {
    instances: Vec<ConfigInstance>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RevisionJson {
    revision: ConfigInstanceRevision,
}

#[async_trait]
impl FileBasedStorageAdapter for LocalFileStorageAdapter {
    async fn get_configs(&self) -> Result<Vec<Config>, GenericStorageError> {
        let path = self.get_configs_datafile_path();
        let content = fs::read_to_string(path)?;
        let v: ConfigJson = serde_json::from_str(&content)?;
        return Ok(v.configs);
    }

    async fn save_configs(&self, configs: Vec<Config>) -> Result<(), GenericStorageError> {
        // Add config to base config file
        let data = serde_json::to_string(&ConfigJson { configs: configs })?;
        let path: String = self.get_configs_datafile_path();
        let mut file = File::create(&path)?;
        Write::write_all(&mut file, data.as_bytes())?;
        Ok(())
    }

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError> {
        let path = self.get_labels_datafile_path();
        let content = fs::read_to_string(path)?;
        let v: LabelJson = serde_json::from_str(&content)?;
        return Ok(v.labels);
    }

    async fn save_labels(&self, labels: Vec<LabelType>) -> Result<(), GenericStorageError> {
        let label_file = self.get_labels_datafile_path();
        let data = serde_json::to_string(&LabelJson { labels: labels })?;
        let mut file = File::create(&label_file)?;
        Write::write_all(&mut file, data.as_bytes())?;
        return Ok(());
    }

    async fn get_config_instance_metadata(&self, config_name: &str) -> Option<Vec<ConfigInstance>> {
        let metadata_dir = self.get_config_instance_metadata_dir();
        let instance_file = format!("{metadata_dir}/{config_name}.json");
        if let Some(content) = fs::read_to_string(instance_file).ok() {
            let v: InstanceJson = serde_json::from_str(&content).unwrap();
            return Some(v.instances);
        }
        return None;
    }

    async fn create_config_instance_dir(
        &self,
        config_name: &str,
    ) -> Result<(), GenericStorageError> {
        let config_instance_dir = self.get_config_instance_dir();
        let config_instance_path = format!("{config_instance_dir}/{config_name}");
        fs::create_dir(&config_instance_path)?;
        return Ok(());
    }

    async fn create_revision_instance_dir(
        &self,
        config_name: &str,
    ) -> Result<(), GenericStorageError> {
        let revision_instance_dir = self.get_instance_revisions_path();
        let revision_instance_path = format!("{revision_instance_dir}/{config_name}");
        fs::create_dir(&revision_instance_path)?;
        return Ok(());
    }

    async fn create_config_instance_data_file(
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

    async fn get_data_by_revision(&self, config_name: &str, revision: &str) -> Option<String> {
        let revision_dir = self.get_instance_revisions_path();
        let revision_path = format!("{revision_dir}/{config_name}/{}", revision);
        println!("Fetching revision {}", revision_path);
        if let Some(content) = fs::read_to_string(revision_path).ok() {
            let revision_data: RevisionJson = serde_json::from_str(&content).unwrap();
            let key = &revision_data.revision.data_key;
            let instance_dir = self.get_config_instance_dir();
            let instance_path = format!("{instance_dir}/{config_name}/{key}");
            println!("Fetching instance data {}", instance_path);
            return fs::read_to_string(instance_path).ok();
        }
        println!("Fetching revision not found");
        return None;
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

    async fn get_revsion(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Option<ConfigInstanceRevision> {
        let dir = self.get_instance_revisions_path();
        let path = format!("{dir}/{config_name}/{revision}");

        println!("checking {} ", path);

        if let Ok(content) = fs::read_to_string(&path) {
            println!("got data {} ", content);
            let data: Option<RevisionJson> = serde_json::from_str(&content).ok();
            return data.map(|r| r.revision);
        } else {
            println!("Failed to load revision file: {revision}");
        }

        return None;
    }

    async fn update_revision_data(
        &self,
        config_name: &str,
        revision: &ConfigInstanceRevision,
    ) -> Result<(), GenericStorageError> {
        let revisions_path = self.get_instance_revisions_path();
        let revision_key = &revision.revision;
        let revision_data = serde_json::to_string(&RevisionJson {
            revision: revision.clone(), // TODO: This does not need to be cloned if we use lifetime annotations
        })?;
        let revision_file_path = format!("{revisions_path}/{config_name}/{revision_key}");
        let mut revision_file = File::create(&revision_file_path)?;
        Write::write_all(&mut revision_file, revision_data.as_bytes())?;
        println!("Created revision file: {}", revision_file_path);
        return Ok(());
    }
}

impl LocalFileStorageAdapter {
    async fn initialize_adapter(&mut self) {
        println!("initializing local storage adapter");

        let yakman_dir = self.get_yakman_dir();
        if !Path::new(&yakman_dir).is_dir() {
            println!("Creating {}", yakman_dir);
            fs::create_dir(&yakman_dir)
                .expect(&format!("Failed to create base dir: {}", yakman_dir));
        }

        let config_file = self.get_configs_datafile_path();
        if !Path::new(&config_file).is_file() {
            println!("Creating {}", config_file);
            let data = serde_json::to_string(&ConfigJson { configs: vec![] })
                .expect("Failed to create configs json");
            let mut file = File::create(&config_file).expect("Failed to create configs file");
            Write::write_all(&mut file, data.as_bytes())
                .expect("Failed to write data to the configs file");
        }

        self.save_labels(vec![])
            .await
            .expect("Failed to create labels file");

        let instance_dir = self.get_config_instance_dir();
        if !Path::new(&instance_dir).is_dir() {
            println!("Creating {}", instance_dir);
            fs::create_dir(&instance_dir)
                .expect(&format!("Failed to create instance dir: {}", instance_dir));
        }

        let revision_dir = self.get_instance_revisions_path();
        if !Path::new(&revision_dir).is_dir() {
            println!("Creating {}", revision_dir);
            fs::create_dir(&revision_dir)
                .expect(&format!("Failed to create revision dir: {}", instance_dir));
        }

        let instance_metadata_dir = self.get_config_instance_metadata_dir();
        if !Path::new(&instance_metadata_dir).is_dir() {
            println!("Creating {}", instance_metadata_dir);
            fs::create_dir(&instance_metadata_dir).expect(&format!(
                "Failed to create instance metadata dir: {}",
                instance_metadata_dir
            ));
        }
    }

    async fn get_config_data_by_labels(
        &self,
        config_name: &str,
        labels: Vec<Label>,
    ) -> Option<String> {
        if let Some(instances) = self.get_config_instance_metadata(config_name).await {
            println!("Found {} instances", instances.len());
            let label_types = self.get_labels().await.unwrap();
            let selected_instance = select_instance(instances, labels, label_types);

            if let Some(instance) = selected_instance {
                return self
                    .get_data_by_revision(config_name, &instance.current_revision)
                    .await;
            }
            println!("No selected instance found");
            return None;
        }
        return None;
    }

    async fn get_config_data(&self, config_name: &str, instance: &str) -> Option<String> {
        if let Some(instances) = self.get_config_instance_metadata(config_name).await {
            println!("Found {} instances", instances.len());

            println!("Search for instance ID {}", instance);
            let selected_instance = instances.iter().find(|i| i.instance == instance);

            if let Some(instance) = selected_instance {
                return self
                    .get_data_by_revision(config_name, &instance.current_revision)
                    .await;
            }
            println!("No selected instance found");
            return None;
        }
        return None;
    }

    async fn create_config_instance(
        &self,
        config_name: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), GenericStorageError> {
        if let Some(mut instances) = self.get_config_instance_metadata(config_name).await {
            let instance = Uuid::new_v4().to_string();
            let revision_key = Uuid::new_v4().to_string();
            let data_key = Uuid::new_v4().to_string();

            // Create new file with data
            self.create_config_instance_data_file(config_name, &data_key, data)
                .await?;

            // Create revision
            let revision = ConfigInstanceRevision {
                revision: String::from(&revision_key),
                data_key: String::from(&data_key),
                labels: labels,
                timestamp_ms: Utc::now().timestamp_millis(),
                approved: false,
            };
            self.update_revision_data(config_name, &revision).await?;

            // Add new instance to instances and update the instance datafile
            instances.push(ConfigInstance {
                config_name: config_name.to_string(),
                instance: instance,
                labels: revision.labels,
                current_revision: String::from(&revision.revision),
                pending_revision: None,
                revisions: vec![revision.revision],
            });
            self.save_instance_metadata(config_name, instances).await?;
            println!("Update instance metadata for config: {}", config_name);

            return Ok(());
        }

        todo!()
        // return Err(Box::new(ConfigNotFoundError {
        //     description: format!("Config not found: {config_name}"),
        // }));
    }

    async fn update_config_instance(
        &self,
        config_name: &str,
        instance: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), GenericStorageError> {
        if let Some(mut instances) = self.get_config_instance_metadata(config_name).await {
            let revision_key = Uuid::new_v4().to_string();
            let data_key = Uuid::new_v4().to_string();

            let base_path = self.path.to_string(); // TODO: replace with helper func

            // Create new file with data
            let data_file_path = format!("{base_path}/{DATA_DIR}/{config_name}/{data_key}");
            let mut data_file = File::create(&data_file_path)?;
            Write::write_all(&mut data_file, data.as_bytes())?;
            println!("Created data file: {}", data_file_path);

            // Create revision
            let revision = ConfigInstanceRevision {
                revision: String::from(&revision_key),
                data_key: String::from(&data_key),
                labels: labels,
                timestamp_ms: Utc::now().timestamp_millis(),
                approved: false,
            };
            self.update_revision_data(config_name, &revision).await?;

            // Update instance data
            if let Some(instance) = instances.iter_mut().find(|inst| inst.instance == instance) {
                instance.pending_revision = Some(String::from(&revision.revision));
                self.save_instance_metadata(config_name, instances).await?;
                println!("Updated instance metadata for config: {config_name}");
                return Ok(());
            } // TODO: Throw a new custom for failed to update config metadata
        }

        todo!()
        // return Err(Box::new(ConfigNotFoundError {
        //     description: format!("Config not found: {config_name}"),
        // }));
    }

    async fn approve_pending_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
    ) -> Result<(), ApproveRevisionError> {
        let mut metadata = match self.get_config_instance_metadata(config_name).await {
            Some(metadata) => metadata,
            None => return Err(ApproveRevisionError::InvalidConfig),
        };

        let mut instance = match metadata.iter_mut().find(|i| i.instance == instance) {
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

        let mut revision_data = match self.get_revsion(config_name, revision).await {
            Some(revision_data) => revision_data,
            None => return Err(ApproveRevisionError::InvalidRevision),
        };

        // if revision_data.approved {
        //     return Err(ApproveRevisionError::AlreadyApproved);
        // }

        revision_data.approved = true;
        self.update_revision_data(config_name, &revision_data)
            .await
            .map_err(|e| ApproveRevisionError::StorageError {
                message: e.to_string(),
            })?;

        instance.current_revision = String::from(revision);
        instance.pending_revision = None;
        instance.labels = revision_data.labels;

        if !instance.revisions.contains(&String::from(revision)) {
            instance.revisions.push(String::from(revision));
        }

        self.save_instance_metadata(config_name, metadata)
            .await
            .map_err(|e| ApproveRevisionError::StorageError {
                message: e.to_string(),
            })?;

        return Ok(());
    }

    async fn get_instance_revisions(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Option<Vec<ConfigInstanceRevision>> {
        let instances = match self.get_config_instance_metadata(&config_name).await {
            Some(value) => value,
            None => return None,
        };

        let instance = match instances.iter().find(|inst| inst.instance == instance) {
            Some(value) => value,
            None => return None,
        };

        println!("found {} revisions", instance.revisions.len());

        let mut revisions: Vec<ConfigInstanceRevision> = vec![];

        for rev in instance.revisions.iter() {
            if let Some(revision) = self.get_revsion(config_name, &rev).await {
                revisions.push(revision);
            }
        }

        return Some(revisions);
    }

    async fn update_instance_current_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
    ) -> Result<(), GenericStorageError> {
        let mut instances = self
            .get_config_instance_metadata(config_name)
            .await
            .unwrap(); // TODO: propagate error

        let mut instance = instances
            .iter_mut()
            .find(|i| i.instance == instance)
            .unwrap(); // TODO: propagate error

        if !instance.revisions.contains(&String::from(revision)) {
            panic!("revision not found!"); // TODO: propagate error
        }
        instance.pending_revision = Some(String::from(revision));

        self.save_instance_metadata(config_name, instances).await?;

        return Ok(());
    }
}

impl LocalFileStorageAdapter {
    fn get_yakman_dir(&self) -> String {
        return format!("{}/{YAK_MAN_DIR}", self.path.as_str());
    }

    fn get_labels_datafile_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/labels.json");
    }

    fn get_configs_datafile_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/configs.json");
    }

    fn get_instance_revisions_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/instance-revisions");
    }

    fn get_config_instance_dir(&self) -> String {
        return format!("{}/{DATA_DIR}", self.path.as_str());
    }

    fn get_config_instance_metadata_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/instance-metadata");
    }
}
