use crate::data_types::{Config, ConfigInstance, Label, LabelType};

// The base storage adapter to be able to load config from external storage

#[async_trait]
pub trait ConfigStorageAdapter: Sync + Send {
    async fn get_configs(&self) -> Vec<Config>;

    async fn get_labels(&self) -> Vec<LabelType>;

    async fn get_config_instance_metadata(&self, id: &str) -> Option<Vec<ConfigInstance>>;

    async fn get_config_data(&self, id: &str, labels: Vec<Label>) -> Option<String>;
}
