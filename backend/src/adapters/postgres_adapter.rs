use sqlx::{postgres::PgPoolOptions, query_as, FromRow, Pool, Postgres};

use yak_man_core::model::{Config, ConfigInstance, Label, LabelType};

use crate::adapters::ConfigStorageAdapter;

use super::utils::select_instance;

pub struct PostgresAdapter {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String,
    pub pool: Option<Pool<Postgres>>, // TODO: make internal only?
}

pub fn create_postgres_adapter() -> impl ConfigStorageAdapter {
    // TODO: get data from env vars
    return PostgresAdapter {
        host: "localhost".to_string(),
        port: 5432,
        username: "postgres".to_string(),
        password: "password".to_string(),
        pool: None,
    };
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

#[derive(Debug, FromRow)]
struct PostgresConfigInstance {
    instance_id: i32,
    config_name: String,
}

#[derive(Debug, FromRow)]
struct PostgresConfigInstanceLabel {
    instance_id: i32,
    label_name: String,
    option: String,
}

#[derive(Debug, FromRow)]
struct PostgresConfigInstanceData {
    data: String,
}

const SELECT_CONFIGS_QUERY: &str = "SELECT name, description FROM YAK_MAN_CONFIG";
const SELECT_LABELS_QUERY: &str = "SELECT name, description FROM YAK_MAN_LABEL";
const SELECT_LABEL_OPTIONS_QUERY: &str = "SELECT option FROM YAK_MAN_LABEL_OPTION where name = $1";

#[async_trait]
impl ConfigStorageAdapter for PostgresAdapter {
    async fn initialize_adapter(&mut self) {
        println!("Initializing Postgres connection pool...");

        let pool = self.create_connnection_pool().await;
        self.pool = Some(pool);

        println!("Created Postgres connection pool successfully");
    }

    async fn get_configs(&self) -> Vec<Config> {
        let pool = self.get_connection().await;

        let select_query = query_as::<Postgres, PostgresConfig>(SELECT_CONFIGS_QUERY);
        let configs = select_query.fetch_all(pool).await.unwrap(); // TODO: safe unwrap

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
        let labels = select_labels_query.fetch_all(pool).await.unwrap(); // TODO: safe unwrap

        if labels.len() == 0 {
            return vec![];
        }

        let mut label_types: Vec<LabelType> = vec![];

        for label in labels {
            let query = query_as::<Postgres, PostgresLabelOption>(SELECT_LABEL_OPTIONS_QUERY);
            let option = query.bind(&label.name).fetch_all(pool).await.unwrap(); // TODO: safe unwrap

            label_types.push(LabelType {
                name: label.name.to_owned(),
                description: label.description,
                priority: 1, // TODO: handle
                options: option.iter().map(|option| option.option.clone()).collect(),
            });
        }

        return label_types;
    }

    async fn get_config_instance_metadata(&self, config_name: &str) -> Option<Vec<ConfigInstance>> {
        let pool = self.get_connection().await;

        let q = "SELECT config_name, instance_id FROM yak_man_instance WHERE config_name = $1";
        let query = query_as::<Postgres, PostgresConfigInstance>(q);
        let data = query.bind(config_name).fetch_all(pool).await.unwrap(); // TODO: safe unwrap

        println!("{:?}", data);

        let mut instances: Vec<ConfigInstance> = vec![];

        for instance in data {
            let q = "SELECT instance_id, label_name, option FROM YAK_MAN_INSTANCE_LABEL WHERE instance_id = $1";
            let query = query_as::<Postgres, PostgresConfigInstanceLabel>(q);
            let labels = query
                .bind(instance.instance_id)
                .fetch_all(pool)
                .await
                .unwrap(); // TODO: safe unwrap

            let labels = labels
                .iter()
                .map(|lbl| Label {
                    label_type: lbl.label_name.to_owned(),
                    value: lbl.option.to_owned(),
                })
                .collect();

            instances.push(ConfigInstance {
                config_name: instance.config_name,
                instance: instance.instance_id.to_string(),
                labels: labels,
            });
        }

        return Some(instances);
    }

    async fn get_config_data(&self, config_name: &str, labels: Vec<Label>) -> Option<String> {
        if let Some(instances) = self.get_config_instance_metadata(config_name).await {
            let label_types = self.get_labels().await;
            let selected_instance: Option<ConfigInstance> =
                select_instance(instances, labels, label_types);

            if let Some(instance) = selected_instance {
                let pool = self.get_connection().await;

                let q = "SELECT data FROM YAK_MAN_INSTANCE WHERE instance_id = $1";
                let query = query_as::<Postgres, PostgresConfigInstanceData>(q);
                let data = query
                    .bind(instance.instance.parse::<i32>().unwrap())
                    .fetch_one(pool)
                    .await
                    .unwrap(); // TODO: safe unwrap

                return Some(data.data);
            } else {
                println!("No selected instance found");
                return None;
            }
        }

        return None;
    }
}

impl PostgresAdapter {
    async fn get_connection(&self) -> &Pool<Postgres> {
        return &self.pool.as_ref().unwrap();
    }

    async fn create_connnection_pool(&self) -> Pool<Postgres> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://postgres:password@localhost") // TODO: use env vars
            .await;

        return pool.unwrap(); // TODO: handle this better
    }
}
