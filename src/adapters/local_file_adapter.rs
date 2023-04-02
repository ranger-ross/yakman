use std::{collections::HashMap, fs};

use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};

use crate::{
    adapters::ConfigStorageAdapter,
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

const CONFIG_MAN_DIR: &str = ".configman"; // TODO: clean up
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
            let mut selected_instance: Option<ConfigInstance> = None;

            for instance in instances {
                if instance.labels == labels {
                    // TODO: Create better comparison logic
                    selected_instance = Some(instance);
                    break;
                }
            }

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

/// labels = selected labels, label_types = all label types avaiable, instances = all instances to select from
fn select_instance(
    instances: Vec<ConfigInstance>,
    labels: Vec<Label>,
    label_types: Vec<LabelType>,
) -> Option<ConfigInstance> {
    let label_type_map: HashMap<String, &LabelType> = label_types
        .iter()
        .map(|label| (label.name.clone(), label.clone()))
        .collect();
    let selected_label_type_map: HashMap<String, &Label> = labels
        .iter()
        .map(|label| (label.label_type.clone(), label.clone()))
        .collect();
    let label_count = labels.len();

    let mut matched_instance: Option<ConfigInstance> = None;
    let mut matched_instance_labels: Vec<&Label> = vec![];

    for instance in instances {
        if instance.labels == labels {
            return Some(instance);
        }

        let mut matched_labels: Vec<&Label> = vec![];

        for label in &instance.labels {
            let label_type = label_type_map.get(&label.label_type).unwrap(); // todo: handle
            let selected_label = selected_label_type_map.get(&label_type.name);
            match selected_label {
                Some(selected_label) => {
                    if selected_label.value == label.value {
                        matched_labels.push(selected_label.to_owned());
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        if label_count > matched_labels.len() {
            // missing label, cannot select
            continue;
        }

        if matched_labels.len() > matched_instance_labels.len() {
            matched_instance = Some(instance);
            matched_instance_labels = matched_labels;
        } else {
            // IF THE MATCHING LABELS ARE THE SAME, CHECK IF THE LABELS ARE HIGHER PRIORITY
            matched_labels.sort_by(|a, b| {
                let a_type = label_type_map.get(&a.label_type).unwrap(); // todo: handle
                let b_type = label_type_map.get(&b.label_type).unwrap(); // todo: handle
                return a_type.priority.cmp(&b_type.priority);
            });
            matched_instance_labels.sort_by(|a, b| {
                let a_type = label_type_map.get(&a.label_type).unwrap(); // todo: handle
                let b_type = label_type_map.get(&b.label_type).unwrap(); // todo: handle
                return a_type.priority.cmp(&b_type.priority);
            });

            for i in 1..matched_labels.len() {

                let lbl = label_type_map.get(&matched_labels.get(i).unwrap().label_type).unwrap(); // todo: handle
                let matched_lbl = label_type_map.get(&matched_instance_labels.get(i).unwrap().label_type).unwrap(); // todo: handle

                if lbl.priority > matched_lbl.priority {
                    matched_instance = Some(instance);
                    matched_instance_labels = matched_labels;
                    break;
                }
                
            }
        }
    }

    return matched_instance;
}
