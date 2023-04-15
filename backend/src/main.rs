mod adapters;
mod utils;

use adapters::{
    errors::{CreateConfigError, CreateLabelError},
    ConfigStorageAdapter,
};
use rocket::{
    http::Status,
    serde::json::{serde_json, Json},
    State,
};
use std::{env, vec};
use utils::raw_query::RawQuery;
use yak_man_core::{
    load_yak_man_settings,
    model::{Config, ConfigInstance, ConfigInstanceRevision, Label, LabelType},
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
            routes![
                configs,
                labels,
                create_label,
                get_instance_by_id,
                data,
                create_new_instance,
                instance,
                create_config,
                update_new_instance,
                get_instance_revisions,
                update_instance_current_revision
            ],
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

#[put("/labels", data = "<data>")]
async fn create_label(data: String, state: &State<StateManager>) -> Status {
    let adapter = state.get_adapter();

    let label_type: Option<LabelType> = serde_json::from_str(&data).ok();

    if let Some(label_type) = label_type {
        return match adapter.create_label(label_type).await {
            Ok(()) => Status::Ok,
            Err(e) => match e {
                CreateLabelError::DuplicateLabelError { name } => Status::BadRequest,
                CreateLabelError::EmptyOptionsError => Status::BadRequest,
                CreateLabelError::InvalidPriorityError { prioity } => Status::BadRequest,
                CreateLabelError::StorageError { message } => {
                    println!("Failed to create label, error: {message}");
                    Status::InternalServerError
                }
            },
        };
    }

    return Status::BadRequest; // Bad input so parse failed
}

#[get("/instances/<id>")]
async fn get_instance_by_id(
    id: &str,
    state: &State<StateManager>,
) -> Option<Json<Vec<ConfigInstance>>> {
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

    println!("Search for config {config_name} with labels: {:?}", labels);

    return adapter.get_config_data_by_labels(config_name, labels).await;
}

#[get("/config/<config_name>/instance/<instance>")]
async fn instance(
    config_name: &str,
    instance: &str,
    state: &State<StateManager>,
) -> Option<String> {
    let adapter = state.get_adapter();
    return adapter.get_config_data(config_name, instance).await;
}

#[put("/config/<config_name>/data", data = "<data>")] // TODO: Rename to /instance
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

#[post("/config/<config_name>/instance/<instance>", data = "<data>")]
async fn update_new_instance(
    config_name: &str,
    instance: &str,
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

    println!("lables {:?}", &labels);

    // TODO: do validation
    // - config exists
    // - labels are valid
    // - not a duplicate?

    adapter
        .update_config_instance(config_name, instance, labels, &data)
        .await
        .unwrap();
}

#[put("/config/<config_name>")]
async fn create_config(config_name: &str, state: &State<StateManager>) -> Status {
    let adapter = state.get_adapter();
    let result = adapter.create_config(config_name).await;

    match result {
        Ok(()) => Status::Ok,
        Err(e) => match e {
            CreateConfigError::StorageError { message } => {
                println!("Failed to create config {config_name}, error: {message}");
                Status::InternalServerError
            }
            CreateConfigError::DuplicateConfigError { name: _ } => Status::BadRequest,
        },
    }
}

#[get("/config/<config_name>/instance/<instance>/revisions")]
async fn get_instance_revisions(
    config_name: &str,
    instance: &str,
    state: &State<StateManager>,
) -> Option<Json<Vec<ConfigInstanceRevision>>> {
    let adapter = state.get_adapter();

    if let Some(data) = adapter.get_instance_revisions(config_name, instance).await {
        return Some(Json(data));
    }
    return None;
}

#[post("/config/<config_name>/instance/<instance>/revision/<revision>/current")]
async fn update_instance_current_revision(
    config_name: &str,
    instance: &str,
    revision: &str,
    state: &State<StateManager>,
) {
    let adapter = state.get_adapter();

    adapter
        .update_instance_current_revision(config_name, instance, revision)
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
