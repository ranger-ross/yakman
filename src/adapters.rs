use crate::data_types::{AppConfig, AppConfigInstance};

// The base storage adapter to be able to load config from external storage
pub trait ConfigStorageAdapter {
    fn load_configs(self) -> Vec<AppConfig>;

    fn load_config(self, id: i32) -> AppConfigInstance;
}

pub struct LocalFileStorageAdapter {
    pub path: String,
}

impl ConfigStorageAdapter for LocalFileStorageAdapter {
    fn load_configs(self) -> Vec<AppConfig> {
        return vec![AppConfig {
            id: 100,
            name: "Testing".to_string(),
        }]; // TODO: fix
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
