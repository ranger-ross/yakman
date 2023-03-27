use crate::data_types::{Config, ConfigInstance, Label, LabelType};

// The base storage adapter to be able to load config from external storage

pub trait ConfigStorageAdapter: Sync + Send {
    fn get_configs(self) -> Vec<Config>;

    fn get_labels(self) -> Vec<LabelType>;

    fn get_config_instance_metadata(self, id: &str) -> Option<Vec<ConfigInstance>>;

    fn get_config_data(self, id: &str, labels: Vec<Label>) -> Option<String>;
}
