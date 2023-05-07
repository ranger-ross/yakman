// use crate::adapters::ConfigStorageAdapter;
// use yak_man_core::model::{Config, ConfigInstance, ConfigInstanceRevision, Label, LabelType};

// extern crate redis;
// use redis::{Commands, Connection, RedisResult};
// use serde::{Deserialize, Serialize};

// use super::{utils::select_instance, CreateConfigError, errors::{CreateLabelError, ApproveRevisionError}};

// #[derive(Debug, Serialize, Deserialize)]
// struct ConfigJson {
//     configs: Vec<Config>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct LabelJson {
//     labels: Vec<LabelType>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct InstanceJson {
//     instances: Vec<ConfigInstance>,
// }

// pub struct RedisStorageAdapter {
//     pub host: String,
//     pub port: i32,
//     pub username: String,
//     pub password: String,
// }

// pub fn create_redis_adapter() -> impl ConfigStorageAdapter {
//     // TODO: use env vars
//     return RedisStorageAdapter {
//         host: "127.0.0.1".to_string(),
//         port: 6379,
//         username: "".to_string(),
//         password: "".to_string(),
//     };
// }

// const REDIS_PREFIX: &str = "CONFIG_MAN_";

// #[async_trait]
// impl ConfigStorageAdapter for RedisStorageAdapter {

// }

// impl RedisStorageAdapter {
//     fn open_connection(&self) -> RedisResult<Connection> {
//         // TODO: Handle Auth
//         let connection_url: String =
//             "redis://".to_string() + self.host.as_str() + ":" + self.port.to_string().as_str();
//         let client = redis::Client::open(connection_url)?;
//         return client.get_connection();
//     }
// }
