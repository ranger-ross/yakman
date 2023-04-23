use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use yak_man_core::model::{Config, ConfigInstance, ConfigInstanceRevision, LabelType};

use super::{FileBasedStorageAdapter, GenericStorageError};

#[derive(Clone)]
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
        let path = self.get_configs_file_path();
        let content = fs::read_to_string(path)?;
        let v: ConfigJson = serde_json::from_str(&content)?;
        return Ok(v.configs);
    }

    async fn save_configs(&self, configs: Vec<Config>) -> Result<(), GenericStorageError> {
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

    async fn save_revision(
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

    async fn create_yakman_required_files(&self) -> Result<(), GenericStorageError> {
        let yakman_dir = self.get_yakman_dir();
        if !Path::new(&yakman_dir).is_dir() {
            println!("Creating {}", yakman_dir);
            fs::create_dir(&yakman_dir)
                .expect(&format!("Failed to create base dir: {}", yakman_dir));
        }

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

        Ok(())
    }

    // Directory modification funcs

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
}

// Helper functions
impl LocalFileStorageAdapter {
    fn get_yakman_dir(&self) -> String {
        return format!("{}/{YAK_MAN_DIR}", self.path.as_str());
    }

    fn get_labels_file_path(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/labels.json");
    }

    fn get_configs_file_path(&self) -> String {
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
