use crate::data_types::{Config, ConfigInstance, Label, LabelType};

pub mod redis_adapter;
pub mod postgres_adapter;
pub mod local_file_adapter;

// The base storage adapter to be able to load config from external storage

#[async_trait]
pub trait ConfigStorageAdapter: Sync + Send {

    async fn initialize_adapter(&mut self);

    async fn get_configs(&self) -> Vec<Config>;

    async fn get_labels(&self) -> Vec<LabelType>;

    async fn get_config_instance_metadata(&self, config_name: &str) -> Option<Vec<ConfigInstance>>;

    async fn get_config_data(&self, config_name: &str, labels: Vec<Label>) -> Option<String>;
}
