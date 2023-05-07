// use sqlx::{postgres::PgPoolOptions, query_as, FromRow, Pool, Postgres};

// use yak_man_core::model::{Config, ConfigInstance, Label, LabelType, ConfigInstanceRevision};

// use crate::adapters::ConfigStorageAdapter;

// use super::{utils::select_instance, CreateConfigError, errors::{CreateLabelError, ApproveRevisionError}};

// pub struct PostgresAdapter {
//     pub host: String,
//     pub port: i32,
//     pub username: String,
//     pub password: String,
//     pub pool: Option<Pool<Postgres>>, // TODO: make internal only?
// }

// pub fn create_postgres_adapter() -> impl ConfigStorageAdapter {
//     // TODO: get data from env vars
//     return PostgresAdapter {
//         host: "localhost".to_string(),
//         port: 5432,
//         username: "postgres".to_string(),
//         password: "password".to_string(),
//         pool: None,
//     };
// }

// #[derive(Debug, FromRow)]
// struct PostgresConfig {
//     name: String,
//     description: String,
// }

// #[derive(Debug, FromRow)]
// struct PostgresLabelType {
//     name: String,
//     description: String,
// }

// #[derive(Debug, FromRow)]
// struct PostgresLabelOption {
//     option: String,
// }

// #[derive(Debug, FromRow)]
// struct PostgresConfigInstance {
//     instance_id: i32,
//     config_name: String,
// }

// #[derive(Debug, FromRow)]
// struct PostgresConfigInstanceLabel {
//     instance_id: i32,
//     label_name: String,
//     option: String,
// }

// #[derive(Debug, FromRow)]
// struct PostgresConfigInstanceData {
//     data: String,
// }

// const SELECT_CONFIGS_QUERY: &str = "SELECT name, description FROM YAK_MAN_CONFIG";
// const SELECT_LABELS_QUERY: &str = "SELECT name, description FROM YAK_MAN_LABEL";
// const SELECT_LABEL_OPTIONS_QUERY: &str = "SELECT option FROM YAK_MAN_LABEL_OPTION where name = $1";

// #[async_trait]
// impl ConfigStorageAdapter for PostgresAdapter {

// }

// impl PostgresAdapter {
//     async fn get_connection(&self) -> &Pool<Postgres> {
//         return &self.pool.as_ref().unwrap();
//     }

//     async fn create_connnection_pool(&self) -> Pool<Postgres> {
//         let pool = PgPoolOptions::new()
//             .max_connections(5)
//             .connect("postgres://postgres:password@localhost") // TODO: use env vars
//             .await;

//         return pool.unwrap(); // TODO: handle this better
//     }
// }
