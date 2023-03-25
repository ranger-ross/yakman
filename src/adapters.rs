use std::fs;

use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};

use crate::{
    config_man_state::ConfigManState,
    data_types::{AppConfig, AppConfigInstance, AppLabel, AppLabelType},
};

// The base storage adapter to be able to load config from external storage
pub trait ConfigStorageAdapter {
    fn get_configs(self) -> Vec<AppConfig>;

    fn get_labels(self) -> Vec<AppLabelType>;

    fn get_config_instance_metadata(self, id: i32) -> Vec<AppConfigInstance>; // TODO: Should we use a String instead of Int for the ID?

    fn get_config_data(self, id: i32, labels: Vec<AppLabel>) -> String;
}

pub struct LocalFileStorageAdapter {
    pub path: String,
}

const config_man_dir: &str = ".configman"; // TODO: clean up
const data_dir: &str = "data"; // TODO: clean up

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

    fn get_config_instance_metadata(self, id: i32) -> Vec<AppConfigInstance> {
        let label_file =
            self.path + "/" + config_man_dir + "/instance-metadata/" + &id.to_string() + ".json";
        let content = fs::read_to_string(label_file).expect("Instance data file not found");
        let v: InstanceJson = serde_json::from_str(&content).unwrap();
        return v.instances;
    }

    fn get_config_data(self, id: i32, labels: Vec<AppLabel>) -> String {
        let base_path = self.path.to_string();
        let instances = self.get_config_instance_metadata(id);

        let mut selected_instance: Option<AppConfigInstance> = None;

        for instance in instances {
            if instance.labels == labels {
                // TODO: Create better comparison logic
                selected_instance = Some(instance);
                break;
            }
        }

        if let Some(instance) = selected_instance {
            let path = base_path + "/config-instances/" + instance.instance_id.as_str();
            println!("Found path {}", path);
            return fs::read_to_string(path).expect("Data file not found");

        } else {
            panic!("No selected instance found");
        }
    }
}
