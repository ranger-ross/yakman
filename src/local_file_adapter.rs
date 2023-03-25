use std::fs;

use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};

use crate::{
    adapters::ConfigStorageAdapter,
    data_types::{AppConfig, AppConfigInstance, AppLabel, AppLabelType},
    instances,
};

pub struct LocalFileStorageAdapter {
    pub path: String,
}

const config_man_dir: &str = ".configman"; // TODO: clean up
const data_dir: &str = "config-instances"; // TODO: clean up

#[derive(Debug, Serialize, Deserialize)]
struct LabelJson {
    labels: Vec<AppLabelType>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigJson {
    configs: Vec<AppConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstanceJson {
    instances: Vec<AppConfigInstance>,
}

impl ConfigStorageAdapter for LocalFileStorageAdapter {
    fn get_configs(self) -> Vec<AppConfig> {
        let content =
            fs::read_to_string(self.path + "/" + config_man_dir + "/configs.json").unwrap();
        let v: ConfigJson = serde_json::from_str(&content).unwrap();
        return v.configs;
    }

    fn get_labels(self) -> Vec<AppLabelType> {
        let label_file = self.path + "/" + config_man_dir + "/labels.json";
        let content = fs::read_to_string(label_file).unwrap();
        let v: LabelJson = serde_json::from_str(&content).unwrap();
        return v.labels;
    }

    fn get_config_instance_metadata(self, id: &str) -> Option<Vec<AppConfigInstance>> {
        let label_file =
            self.path + "/" + config_man_dir + "/instance-metadata/" + &id.to_string() + ".json";
        if let Some(content) = fs::read_to_string(label_file).ok() {
            let v: InstanceJson = serde_json::from_str(&content).unwrap();
            return Some(v.instances);
        }
        return None;
    }

    fn get_config_data(self, id: &str, labels: Vec<AppLabel>) -> Option<String> {
        let base_path = self.path.to_string();
        if let Some(instances) = self.get_config_instance_metadata(id) {
            let mut selected_instance: Option<AppConfigInstance> = None;

            for instance in instances {
                if instance.labels == labels {
                    // TODO: Create better comparison logic
                    selected_instance = Some(instance);
                    break;
                }
            }

            if let Some(instance) = selected_instance {
                let path = base_path + "/" + data_dir + "/" + instance.instance_id.as_str();
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
