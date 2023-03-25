use crate::data_types::{AppConfig, AppConfigInstance, AppLabel, AppLabelType};

// The base storage adapter to be able to load config from external storage
pub trait ConfigStorageAdapter {
    fn get_configs(self) -> Vec<AppConfig>;

    fn get_labels(self) -> Vec<AppLabelType>;

    fn get_config_instance_metadata(self, id: &str) -> Option<Vec<AppConfigInstance>>;

    fn get_config_data(self, id: &str, labels: Vec<AppLabel>) -> Option<String>;
}
