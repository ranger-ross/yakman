use std::error::Error;

use sqlx::{postgres::PgPoolOptions, FromRow, Pool, Postgres};

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

#[derive(FromRow)]
struct PostgresConfig {
    pub name: String,
    description: String,
}

#[async_trait]
impl ConfigStorageAdapter for PostgresAdapter {
    async fn get_configs(&self) -> Vec<Config> {
        let pool = self.get_connection().await;

        let select_query = sqlx::query_as::<Postgres, PostgresConfig>(
            "SELECT name, description FROM CONFIG_MAN_CONFIG",
        );
        let configs = select_query.fetch_all(&pool).await.unwrap(); // TODO: safe unwrap

        return configs
            .iter()
            .map(|config| Config {
                name: config.name.clone(), // TODO: find better way to do this?
                description: config.description.clone(),
            })
            .collect();
    }

    async fn get_labels(&self) -> Vec<LabelType> {
        todo!()
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
