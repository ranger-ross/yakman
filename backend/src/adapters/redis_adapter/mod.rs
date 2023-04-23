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
//     async fn initialize_adapter(&mut self) {
//         println!("init");
//     }

//     // async fn get_configs(&self) -> Vec<Config> {
//     //     let mut connection = self.open_connection().expect("Failed to connect to redis");

//     //     let configs: String = connection
//     //         .get(format!("{REDIS_PREFIX}CONFIGS"))
//     //         .expect("Failed to load configs from redis");

//     //     let v: ConfigJson = serde_json::from_str(&configs).unwrap();
//     //     return v.configs;
//     // }

//     async fn get_labels(&self) -> Vec<LabelType> {
//         let mut connection = self.open_connection().expect("Failed to connect to redis");

//         let configs: String = connection
//             .get(format!("{REDIS_PREFIX}LABELS"))
//             .expect("Failed to load configs from redis");

//         let v: LabelJson = serde_json::from_str(&configs).unwrap();
//         return v.labels;
//     }

//     async fn get_config_instance_metadata(&self, config_name: &str) -> Option<Vec<ConfigInstance>> {
//         let mut connection = self.open_connection().expect("Failed to connect to redis");

//         let instance: Option<String> = connection
//             .get(format!("{REDIS_PREFIX}INSTANCE_META_{config_name}"))
//             .ok();

//         if let Some(instance) = instance {
//             let v: InstanceJson = serde_json::from_str(&instance).unwrap();
//             return Some(v.instances);
//         }
//         return None;
//     }

//     async fn get_config_data_by_labels(
//         &self,
//         config_name: &str,
//         labels: Vec<Label>,
//     ) -> Option<String> {
//         if let Some(instances) = self.get_config_instance_metadata(config_name).await {
//             let label_types = self.get_labels().await;
//             let selected_instance: Option<ConfigInstance> =
//                 select_instance(instances, labels, label_types);
//             if let Some(instance) = selected_instance {
//                 let mut connection = self.open_connection().expect("Failed to connect to redis");

//                 let path = format!("{REDIS_PREFIX}INSTANCE_{}", instance.instance.as_str());
//                 println!("Found path {}", path);
//                 return connection.get(path).ok();
//             } else {
//                 println!("No selected instance found");
//                 return None;
//             }
//         }
//         return None;
//     }

//     async fn get_config_data(&self, config_name: &str, instance: &str) -> Option<String> {
//         todo!()
//     }

//     async fn create_config_instance(
//         &self,
//         config_name: &str,
//         labels: Vec<Label>,
//         data: &str,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         todo!();
//     }

//     async fn update_config_instance(
//         &self,
//         config_name: &str,
//         instance: &str,
//         labels: Vec<Label>,
//         data: &str,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         todo!();
//     }

//     async fn create_config(&self, config_name: &str) -> Result<(), CreateConfigError> {
//         todo!()
//     }

//     async fn create_label(&self, label: LabelType) -> Result<(), CreateLabelError> {
//         todo!()
//     }

//     async fn get_instance_revisions(
//         &self,
//         config_name: &str,
//         instance: &str,
//     ) -> Option<Vec<ConfigInstanceRevision>> {
//         todo!()
//     }

//     async fn update_instance_current_revision(&self, config_name: &str, instance: &str, revision: &str) -> Result<(), Box<dyn std::error::Error>> {
//         todo!()
//     }

//     async fn approve_pending_instance_revision(
//         &self,
//         config_name: &str,
//         instance: &str,
//         revision: &str,
//     ) -> Result<(), ApproveRevisionError> {
//         todo!()
//     }

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
