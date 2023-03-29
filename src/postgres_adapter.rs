use std::error::Error;

use sqlx::{postgres::PgPoolOptions, query_as, FromRow, Pool, Postgres};

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

#[derive(Debug, FromRow)]
struct PostgresConfig {
    name: String,
    description: String,
}

#[derive(Debug, FromRow)]
struct PostgresLabelType {
    name: String,
    description: String,
}

#[derive(Debug, FromRow)]
struct PostgresLabelOption {
    option: String,
}

const SELECT_CONFIGS_QUERY: &str = "SELECT name, description FROM CONFIG_MAN_CONFIG";
const SELECT_LABELS_QUERY: &str = "SELECT name, description FROM CONFIG_MAN_LABEL";
const SELECT_LABEL_OPTIONS_QUERY: &str =
    "SELECT option FROM CONFIG_MAN_LABEL_OPTION where name = ?";

#[async_trait]
impl ConfigStorageAdapter for PostgresAdapter {
    async fn get_configs(&self) -> Vec<Config> {
        let pool = self.get_connection().await;

        let select_query = query_as::<Postgres, PostgresConfig>(SELECT_CONFIGS_QUERY);
        let configs = select_query.fetch_all(&pool).await.unwrap(); // TODO: safe unwrap

        return configs
            .iter()
            .map(|config| Config {
                name: config.name.to_owned(),
                description: config.description.to_owned(),
            })
            .collect();
    }

    async fn get_labels(&self) -> Vec<LabelType> {
        let pool = self.get_connection().await;

        let select_labels_query = query_as::<Postgres, PostgresLabelType>(SELECT_LABELS_QUERY);
        let labels = select_labels_query.fetch_all(&pool).await.unwrap(); // TODO: safe unwrap

        if labels.len() == 0 {
            return vec![];
        }

        let mut label_types: Vec<LabelType> = vec![];

        for label in labels {
            label_types.push(LabelType {
                name: label.name.to_owned(),
                description: label.description,
                options: vec![], // TODO: Fetch the rest of the options
            });
        }

        return label_types;
    }

    async fn get_config_instance_metadata(&self, id: &str) -> Option<Vec<ConfigInstance>> {
        todo!()
    }

    async fn get_config_data(&self, id: &str, labels: Vec<Label>) -> Option<String> {
        todo!()
    }
}

impl PostgresAdapter {
    async fn get_connection(&self) -> Pool<Postgres> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://postgres:password@localhost")
            .await;

        return pool.unwrap(); // TODO: handle this better
    }

    async fn create_connnection_pool(self) {
        todo!()
    }
}
