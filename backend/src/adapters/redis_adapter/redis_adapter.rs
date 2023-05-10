extern crate redis;
use super::KVStorageAdapter;
use crate::adapters::errors::GenericStorageError;
use async_trait::async_trait;
use log::warn;
use redis::{Commands, Connection, RedisResult};
use serde::{Deserialize, Serialize};
use yak_man_core::model::{
    Config, ConfigInstance, ConfigInstanceRevision, LabelType, YakManProject, YakManUser,
    YakManUserDetails,
};

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

#[async_trait]
impl KVStorageAdapter for RedisStorageAdapter {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError> {
        todo!()
    }

    async fn save_projects(&self, projects: Vec<YakManProject>) -> Result<(), GenericStorageError> {
        todo!()
    }

    async fn get_configs(&self) -> Result<Vec<Config>, GenericStorageError> {
        todo!()
    }

    async fn get_configs_by_project_uuid(
        &self,
        project_uuid: String,
    ) -> Result<Vec<Config>, GenericStorageError> {
        todo!()
    }

    async fn save_configs(&self, configs: Vec<Config>) -> Result<(), GenericStorageError> {
        todo!()
    }

    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError> {
        todo!()
    }

    async fn save_labels(&self, labels: Vec<LabelType>) -> Result<(), GenericStorageError> {
        todo!()
    }

    async fn get_instance_metadata(
        &self,
        config_name: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, GenericStorageError> {
        todo!()
    }

    async fn get_instance_data(
        &self,
        config_name: &str,
        data_key: &str,
    ) -> Result<String, GenericStorageError> {
        todo!()
    }

    async fn save_instance_data(
        &self,
        config_name: &str,
        data_key: &str,
        data: &str,
    ) -> Result<(), GenericStorageError> {
        todo!()
    }

    async fn save_instance_metadata(
        &self,
        config_name: &str,
        instances: Vec<ConfigInstance>,
    ) -> Result<(), GenericStorageError> {
        todo!()
    }

    async fn get_revsion(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<Option<ConfigInstanceRevision>, GenericStorageError> {
        todo!()
    }

    async fn save_revision(
        &self,
        config_name: &str,
        revision: &ConfigInstanceRevision,
    ) -> Result<(), GenericStorageError> {
        todo!()
    }

    async fn create_config_instance_dir(
        &self,
        config_name: &str,
    ) -> Result<(), GenericStorageError> {
        todo!()
    }

    async fn create_revision_instance_dir(
        &self,
        config_name: &str,
    ) -> Result<(), GenericStorageError> {
        todo!()
    }

    async fn get_users(&self) -> Result<Vec<YakManUser>, GenericStorageError> {
        todo!()
    }

    async fn get_user(&self, id: &str) -> Result<Option<YakManUser>, GenericStorageError> {
        todo!()
    }

    async fn get_user_details(
        &self,
        uuid: &str,
    ) -> Result<Option<YakManUserDetails>, GenericStorageError> {
        todo!()
    }

    async fn save_users(&self, users: Vec<YakManUser>) -> Result<(), GenericStorageError> {
        todo!()
    }

    async fn create_yakman_required_files(&self) -> Result<(), GenericStorageError> {

        warn!("Redis adapter not yet setting up");

        Ok(())
    }
}

impl RedisStorageAdapter {
    fn open_connection(&self) -> RedisResult<Connection> {
        // TODO: Handle Auth
        let connection_url: String =
            "redis://".to_string() + self.host.as_str() + ":" + self.port.to_string().as_str();
        let client = redis::Client::open(connection_url)?;
        return client.get_connection();
    }
}
