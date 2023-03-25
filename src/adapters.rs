use crate::data_types::{AppConfig, AppConfigInstance, AppLabel, AppLabelType};

// The base storage adapter to be able to load config from external storage
pub trait ConfigStorageAdapter {
    fn get_configs(self) -> Vec<AppConfig>;

    fn get_labels(self) -> Vec<AppLabelType>;

    fn get_config_instance_metadata(self, id: i32) -> Vec<AppConfigInstance>; // TODO: Should we use a String instead of Int for the ID?

    fn get_config_data(self, id: i32, labels: Vec<AppLabel>) -> String;
}
