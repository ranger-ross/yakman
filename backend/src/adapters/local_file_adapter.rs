use std::{cmp::Ordering, collections::HashMap, fs};

use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};

use crate::{
    adapters::{ConfigStorageAdapter, utils::select_instance},
    data_types::{Config, ConfigInstance, Label, LabelType},
};

pub struct LocalFileStorageAdapter {
    pub path: String,
}

pub fn create_local_file_adapter() -> impl ConfigStorageAdapter {
    return LocalFileStorageAdapter {
        path: "/home/ross/projects/config-manager/testing-directory".to_string(),
    };
}

const CONFIG_MAN_DIR: &str = ".yakman"; // TODO: clean up
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

#[async_trait]
impl ConfigStorageAdapter for LocalFileStorageAdapter {
    async fn initialize_adapter(&mut self) {
        println!("init");
    }

    async fn get_configs(&self) -> Vec<Config> {
        let path = format!("{}/{CONFIG_MAN_DIR}/configs.json", self.path.as_str());
        let content = fs::read_to_string(path).unwrap();
        let v: ConfigJson = serde_json::from_str(&content).unwrap();
        return v.configs;
    }

    async fn get_labels(&self) -> Vec<LabelType> {
        let path = format!("{}/{CONFIG_MAN_DIR}/labels.json", self.path.as_str());
        let content = fs::read_to_string(path).unwrap();
        let v: LabelJson = serde_json::from_str(&content).unwrap();
        return v.labels;
    }

    async fn get_config_instance_metadata(&self, config_name: &str) -> Option<Vec<ConfigInstance>> {
        let base_path = self.path.as_str();
        let label_file =
            format!("{base_path}/{CONFIG_MAN_DIR}/instance-metadata/{config_name}.json");
        if let Some(content) = fs::read_to_string(label_file).ok() {
            let v: InstanceJson = serde_json::from_str(&content).unwrap();
            return Some(v.instances);
        }
        return None;
    }

    async fn get_config_data(&self, config_name: &str, labels: Vec<Label>) -> Option<String> {
        let base_path = self.path.to_string();
        if let Some(instances) = self.get_config_instance_metadata(config_name).await {
            println!("Found {} instances", instances.len());
            let label_types = self.get_labels().await;
            let selected_instance: Option<ConfigInstance> =
                select_instance(instances, labels, label_types);

            if let Some(instance) = selected_instance {
                let path = format!("{base_path}/{DATA_DIR}/{}", instance.instance.as_str());
                println!("Found path {}", path);
                return fs::read_to_string(path).ok();
            } else {
                println!("No selected instance found");
                return None;
            }
        }
        return None;
    }
}
