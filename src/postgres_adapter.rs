use std::error::Error;

use sqlx::postgres::PgPoolOptions;

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

#[async_trait]
impl ConfigStorageAdapter for PostgresAdapter {
    async fn get_configs(&self) -> Vec<Config> {

        // let x = ross_test().await?;

        todo!()
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

async fn ross_test() -> Result<i32, Box<dyn Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:password@localhost/test")
        .await?;

    return Ok(1);
}

