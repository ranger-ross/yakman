use std::{
    error::Error,
    fmt,
    fs::{self, File},
    io::Write,
    path::Path,
};

use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};

use uuid::Uuid;
use yak_man_core::model::{Config, ConfigInstance, Label, LabelType};

use crate::adapters::{utils::select_instance, ConfigStorageAdapter};

pub struct LocalFileStorageAdapter {
    pub path: String,
}

pub fn create_local_file_adapter() -> impl ConfigStorageAdapter {
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

#[async_trait]
impl ConfigStorageAdapter for LocalFileStorageAdapter {
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

        let label_file = self.get_labels_datafile_path();
        if !Path::new(&label_file).is_file() {
            println!("Creating {}", label_file);
            let data = serde_json::to_string(&LabelJson { labels: vec![] })
                .expect("Failed to create labels json");
            let mut file = File::create(&label_file).expect("Failed to create labels file");
            Write::write_all(&mut file, data.as_bytes())
                .expect("Failed to write data to the labels file");
        }

        let instance_dir = self.get_config_instance_dir();
        if !Path::new(&instance_dir).is_dir() {
            println!("Creating {}", instance_dir);
            fs::create_dir(&instance_dir)
                .expect(&format!("Failed to create instance dir: {}", instance_dir));
        }


        let instance_metadata_dir = self.get_config_instance_metadata_dir();
        if !Path::new(&instance_metadata_dir).is_dir() {
            println!("Creating {}", instance_metadata_dir);
            fs::create_dir(&instance_metadata_dir)
                .expect(&format!("Failed to create instance metadata dir: {}", instance_metadata_dir));
        }

    }

    async fn get_configs(&self) -> Vec<Config> {
        let path = self.get_configs_datafile_path();
        let content = fs::read_to_string(path).unwrap();
        let v: ConfigJson = serde_json::from_str(&content).unwrap();
        return v.configs;
    }

    async fn get_labels(&self) -> Vec<LabelType> {
        let path = self.get_labels_datafile_path();
        let content = fs::read_to_string(path).unwrap();
        let v: LabelJson = serde_json::from_str(&content).unwrap();
        return v.labels;
    }

    async fn get_config_instance_metadata(&self, config_name: &str) -> Option<Vec<ConfigInstance>> {
        let base_path = self.path.as_str();
        let instance_file =
            format!("{base_path}/{YAK_MAN_DIR}/instance-metadata/{config_name}.json");
        if let Some(content) = fs::read_to_string(instance_file).ok() {
            let v: InstanceJson = serde_json::from_str(&content).unwrap();
            return Some(v.instances);
        }
        return None;
    }

    async fn get_config_data_by_labels(
        &self,
        config_name: &str,
        labels: Vec<Label>,
    ) -> Option<String> {
        let base_path = self.path.to_string();
        if let Some(instances) = self.get_config_instance_metadata(config_name).await {
            println!("Found {} instances", instances.len());
            let label_types = self.get_labels().await;
            let selected_instance: Option<ConfigInstance> =
                select_instance(instances, labels, label_types);

            if let Some(instance) = selected_instance {
                let path = format!(
                    "{base_path}/{DATA_DIR}/{config_name}/{}",
                    instance.instance.as_str()
                );
                println!("Found path {}", path);
                return fs::read_to_string(path).ok();
            } else {
                println!("No selected instance found");
                return None;
            }
        }
        return None;
    }

    async fn get_config_data(&self, config_name: &str, instance: &str) -> Option<String> {
        if let Some(instances) = self.get_config_instance_metadata(config_name).await {
            println!("Found {} instances", instances.len());

            println!("Search for instance ID {}", instance);
            let selected_instance = instances.iter().find(|i| i.instance == instance);

            if let Some(instance) = selected_instance {
                let instance_dir = self.get_config_instance_dir();
                let path = format!(
                    "{instance_dir}/{config_name}/{}",
                    instance.instance.as_str()
                );
                println!("Found path {}", path);
                return fs::read_to_string(path).ok();
            } else {
                println!("No selected instance found");
                return None;
            }
        }
        return None;
    }

    async fn create_config_instance(
        &self,
        config_name: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut instances) = self.get_config_instance_metadata(config_name).await {
            let id = Uuid::new_v4().to_string();
            let base_path = self.path.to_string();

            // Create new file with data
            let file_name = format!("{base_path}/{DATA_DIR}/{config_name}/{id}");
            println!("{file_name}");
            let mut file = File::create(&file_name)?;
            Write::write_all(&mut file, data.as_bytes())?;

            // Add new instance to instances and update the instance datafile
            instances.push(ConfigInstance {
                config_name: config_name.to_string(),
                instance: id,
                labels: labels,
            });
            self.update_instance_metadata(config_name, instances)
                .await?;

            return Ok(());
        }

        return Err(Box::new(ConfigNotFoundError {
            description: format!("Config not found: {config_name}"),
        }));
    }

    async fn create_config(&self, config_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut configs = self.get_configs().await;

        // TODO: Check for dups

        configs.push(Config {
            name: String::from(config_name),
            description: String::from(""), // TODO: support descriptions?
        });

        // Create instance metadata file
        let instace_metadata: Vec<ConfigInstance> = vec![];
        let data = serde_json::to_string(&InstanceJson {
            instances: instace_metadata,
        })?;
        let path = format!(
            "{}/{YAK_MAN_DIR}/instance-metadata/{}.json",
            self.path.as_str(),
            config_name
        );
        let mut file = File::create(&path)?;
        Write::write_all(&mut file, data.as_bytes())?;

        // Create config instances directory
        let config_instance_dir = self.get_config_instance_dir();
        println!("Creating dir {config_instance_dir}");
        fs::create_dir(format!("{config_instance_dir}/{config_name}"))?;

        // Add config to base config file
        let data = serde_json::to_string(&ConfigJson { configs: configs })?;
        let path = self.get_configs_datafile_path();
        let mut file = File::create(&path)?;
        Write::write_all(&mut file, data.as_bytes())?;

        Ok(())
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

    fn get_config_instance_dir(&self) -> String {
        return format!("{}/{DATA_DIR}", self.path.as_str());
    }

    fn get_config_instance_metadata_dir(&self) -> String {
        let yakman_dir = self.get_yakman_dir();
        return format!("{yakman_dir}/instance-metadata");
    }

    async fn update_instance_metadata(
        &self,
        config_name: &str,
        instances: Vec<ConfigInstance>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let base_path = self.path.as_str();
        let instance_file =
            format!("{base_path}/{YAK_MAN_DIR}/instance-metadata/{config_name}.json");
        let data = serde_json::to_string(&InstanceJson {
            instances: instances,
        })?;

        let mut file = File::create(&instance_file)?;
        Write::write_all(&mut file, data.as_bytes())?;

        Ok(())
    }
}

// TODO: Refactor to base adapter ?
#[derive(Debug)]
struct ConfigNotFoundError {
    description: String,
}

impl fmt::Display for ConfigNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::error::Error for ConfigNotFoundError {
    fn description(&self) -> &str {
        &self.description
    }
}
