mod adapters;
mod utils;

use adapters::ConfigStorageAdapter;
use rocket::{serde::json::Json, State};
use std::{env, vec};
use utils::raw_query::RawQuery;
use yak_man_core::{
    load_yak_man_settings,
    model::{Config, ConfigInstance, Label, LabelType},
};

use crate::adapters::{
    local_file_adapter::create_local_file_adapter, postgres_adapter::create_postgres_adapter,
    redis_adapter::create_redis_adapter,
};

#[macro_use]
extern crate rocket;

struct StateManager {
    adapter: Box<dyn ConfigStorageAdapter>,
}

impl StateManager {
    fn get_adapter(&self) -> &dyn ConfigStorageAdapter {
        return self.adapter.as_ref();
    }
}

#[launch]
async fn rocket() -> _ {
    let settings = load_yak_man_settings();
    println!("Settings: {:?}", settings);

    let mut adapter = create_adapter();

    adapter.initialize_adapter().await;

    rocket::build()
        .manage(StateManager { adapter: adapter })
        .mount(
            "/",
            routes![configs, labels, instances, data, create_new_instance, instance],
        )
}

#[get("/configs")]
async fn configs(state: &State<StateManager>) -> Json<Vec<Config>> {
    let adapter = state.get_adapter();
    return Json(adapter.get_configs().await);
}

#[get("/labels")]
async fn labels(state: &State<StateManager>) -> Json<Vec<LabelType>> {
    let adapter = state.get_adapter();
    return Json(adapter.get_labels().await);
}

#[get("/instances/<id>")]
async fn instances(id: &str, state: &State<StateManager>) -> Option<Json<Vec<ConfigInstance>>> {
    let adapter = state.get_adapter();
    return match adapter.get_config_instance_metadata(id).await {
        Some(data) => Some(Json(data)),
        None => None,
    };
}

// TODO: Standardize REST endpoint naming

#[get("/config/<config_name>/instance")]
async fn data(config_name: &str, query: RawQuery, state: &State<StateManager>) -> Option<String> {
    let adapter = state.get_adapter();

    let labels: Vec<Label> = query
        .params
        .iter()
        .map(|param| Label {
            label_type: param.0.to_string(),
            value: param.1.to_string(),
        })
        .collect();

    println!(
        "Search for config {} with labels: {:?}",
        config_name, labels
    );

    return adapter.get_config_data_by_labels(config_name, labels).await;
}

#[get("/config/<config_name>/instance/<instance>")]
async fn instance(config_name: &str, instance: &str, state: &State<StateManager>) -> Option<String> {
    let adapter = state.get_adapter();
    return adapter.get_config_data(config_name, instance).await;
}


#[put("/config/<config_name>/data", data = "<data>")]
async fn create_new_instance(
    config_name: &str,
    query: RawQuery,
    data: String,
    state: &State<StateManager>,
) {
    let adapter = state.get_adapter();

    let labels: Vec<Label> = query
        .params
        .iter()
        .map(|param| Label {
            label_type: param.0.to_string(),
            value: param.1.to_string(),
        })
        .collect();

    // TODO: do validation
    // - config exists
    // - labels are valid
    // - not a duplicate?

    adapter
        .create_config_instance(config_name, labels, &data)
        .await
        .unwrap();
}

fn create_adapter() -> Box<dyn ConfigStorageAdapter> {
    let adapter_name = env::var("YAKMAN_ADAPTER").expect("$YAKMAN_ADAPTER is not set");

    return match adapter_name.as_str() {
        "REDIS" => Box::new(create_redis_adapter()),
        "POSTGRES" => Box::new(create_postgres_adapter()),
        "LOCAL_FILE_SYSTEM" => Box::new(create_local_file_adapter()),
        _ => panic!("Unsupported adapter {adapter_name}"),
    };
}
