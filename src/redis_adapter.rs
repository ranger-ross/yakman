use crate::{
    adapters::ConfigStorageAdapter,
    data_types::{Config, ConfigInstance, Label, LabelType},
};

extern crate redis;
use redis::{Commands, Connection, RedisResult};
use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ConfigJson {
    configs: Vec<Config>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LabelJson {
    labels: Vec<LabelType>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstanceJson {
    instances: Vec<ConfigInstance>,
}

pub struct RedisStorageAdapter {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String,
}

const REDIS_PREFIX: &str = "CONFIG_MAN_";

impl ConfigStorageAdapter for RedisStorageAdapter {
    fn get_configs(&self) -> Vec<Config> {
        let mut connection = open_connection(&self).expect("Failed to connect to redis");

        let configs: String = connection
            .get(format!("{REDIS_PREFIX}CONFIGS"))
            .expect("Failed to load configs from redis");

        let v: ConfigJson = serde_json::from_str(&configs).unwrap();
        return v.configs;
    }

    fn get_labels(&self) -> Vec<LabelType> {
        let mut connection = open_connection(&self).expect("Failed to connect to redis");

        let configs: String = connection
            .get(format!("{REDIS_PREFIX}LABELS"))
            .expect("Failed to load configs from redis");

        let v: LabelJson = serde_json::from_str(&configs).unwrap();
        return v.labels;
    }

    fn get_config_instance_metadata(&self, id: &str) -> Option<Vec<ConfigInstance>> {
        let mut connection = open_connection(&self).expect("Failed to connect to redis");

        let instance: Option<String> = connection
            .get(format!("{REDIS_PREFIX}INSTANCE_META_{id}"))
            .ok();

        if let Some(instance) = instance {
            let v: InstanceJson = serde_json::from_str(&instance).unwrap();
            return Some(v.instances);
        }
        return None;
    }

    fn get_config_data(&self, id: &str, labels: Vec<Label>) -> Option<String> {
        if let Some(instances) = self.get_config_instance_metadata(id) {
            let mut selected_instance: Option<ConfigInstance> = None;

            for instance in instances {
                if instance.labels == labels {
                    // TODO: Create better comparison logic
                    selected_instance = Some(instance);
                    break;
                }
            }

            if let Some(instance) = selected_instance {
                let mut connection = open_connection(&self).expect("Failed to connect to redis");

                let path = format!("{REDIS_PREFIX}INSTANCE_{}", instance.instance_id.as_str());
                println!("Found path {}", path);
                return connection.get(path).ok();
            } else {
                println!("No selected instance found");
                return None;
            }
        }
        return None;
    }
}

fn open_connection(adapter: &RedisStorageAdapter) -> RedisResult<Connection> {
    // TODO: Handle Auth
    let connection_url: String =
        "redis://".to_string() + adapter.host.as_str() + ":" + adapter.port.to_string().as_str();
    let client = redis::Client::open(connection_url)?;
    return client.get_connection();
}
