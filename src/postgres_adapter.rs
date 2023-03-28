use crate::{
    adapters::ConfigStorageAdapter,
    data_types::{Config, ConfigInstance, Label, LabelType},
};

pub struct PostgresAdapter {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String,
}

impl ConfigStorageAdapter for PostgresAdapter {
    fn get_configs(&self) -> Vec<Config> {
        todo!()
    }

    fn get_labels(&self) -> Vec<LabelType> {
        todo!()
    }

    fn get_config_instance_metadata(&self, id: &str) -> Option<Vec<ConfigInstance>> {
        todo!()
    }

    fn get_config_data(&self, id: &str, labels: Vec<Label>) -> Option<String> {
        todo!()
    }
}
