use std::fs;

use rocket::serde::json::{serde_json, Value};

use crate::data_types::{AppConfig, AppConfigInstance, ConfigManState};

// The base storage adapter to be able to load config from external storage
pub trait ConfigStorageAdapter {
    fn load_configs(self) -> Vec<AppConfig>;

    fn load_config(self, id: i32) -> AppConfigInstance;
}

pub struct LocalFileStorageAdapter {
    pub path: String,
}

const config_man_dir: &str = ".configman"; // TODO: clean up
const data_dir: &str = "data"; // TODO: clean up

impl ConfigStorageAdapter for LocalFileStorageAdapter {
    fn load_configs(self) -> Vec<AppConfig> {
        let content = fs::read_to_string(self.path + "/" + config_man_dir + "/state.json").unwrap();

        let v: ConfigManState = serde_json::from_str(&content).unwrap();

        println!("{:?}", v);

        return v.configs;
    }

    fn load_config(self, id: i32) -> AppConfigInstance {
        // TODO: Fix
        return AppConfigInstance {
            config: AppConfig {
                id: 100,
                name: "FirstConfig".to_string(),
            },
            content: "this is my config data".to_string(),
            labels: vec![],
        };
    }
}
