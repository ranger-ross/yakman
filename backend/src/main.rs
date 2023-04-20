mod adapters;
mod file_based_storage_service;
mod utils;

use adapters::{
    errors::{CreateConfigError, CreateLabelError},
    ConfigStorageAdapter, FileBasedStorageAdapter,
};
use file_based_storage_service::{FileBasedStorageService, StorageService};
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
    local_file_adapter::create_local_file_adapter, 
    // postgres_adapter::create_postgres_adapter,
    // redis_adapter::create_redis_adapter,
};

#[macro_use]
extern crate rocket;

struct StateManager {
    service: Box<dyn StorageService>,
}

impl StateManager {
    fn get_service(&self) -> &dyn StorageService {
        return self.service.as_ref();
    }
}

#[launch]
async fn rocket() -> _ {
    let settings = load_yak_man_settings();
    println!("Settings: {:?}", settings);

    // let mut adapter = create_adapter();

    let service = create_service();

    // service.adapter.initialize_adapter().await; // TODO: fix

    rocket::build()
        .manage(StateManager {
            service: Box::new(service),
        })
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
                // create_config,
                // update_new_instance,
                // get_instance_revisions,
                // update_instance_current_revision,
                // approve_pending_instance_revision
            ],
        )
}

#[get("/configs")]
async fn configs(state: &State<StateManager>) -> Json<Vec<Config>> {
    let service = state.get_service();
    return Json(service.get_configs().await.unwrap()); // TODO: Handle err
}

#[get("/labels")]
async fn labels(state: &State<StateManager>) -> Json<Vec<LabelType>> {
    let service = state.get_service();
    return Json(service.get_labels().await.unwrap());
}

#[put("/labels", data = "<data>")]
async fn create_label(data: String, state: &State<StateManager>) -> Status {
    let service = state.get_service();

    let label_type: Option<LabelType> = serde_json::from_str(&data).ok();

    if let Some(label_type) = label_type {
        return match service.create_label(label_type).await {
            Ok(()) => Status::Ok,
            Err(e) => match e {
                CreateLabelError::DuplicateLabelError { name: _ } => Status::BadRequest,
                CreateLabelError::EmptyOptionsError => Status::BadRequest,
                CreateLabelError::InvalidPriorityError { prioity: _ } => Status::BadRequest,
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
    let service = state.get_service();
    return match service.get_config_instance_metadata(id).await.unwrap() {
        Some(data) => Some(Json(data)),
        None => None,
    };
}

// // TODO: Standardize REST endpoint naming

#[get("/config/<config_name>/instance")]
async fn data(config_name: &str, query: RawQuery, state: &State<StateManager>) -> Option<String> {
    let service = state.get_service();

    let labels: Vec<Label> = query
        .params
        .iter()
        .map(|param| Label {
            label_type: param.0.to_string(),
            value: param.1.to_string(),
        })
        .collect();

    println!("Search for config {config_name} with labels: {:?}", labels);

    return service.get_config_data_by_labels(config_name, labels).await.unwrap();
}

#[get("/config/<config_name>/instance/<instance>")]
async fn instance(
    config_name: &str,
    instance: &str,
    state: &State<StateManager>,
) -> Option<String> {
    let service = state.get_service();
    return service.get_config_data(config_name, instance).await.unwrap();
}

#[put("/config/<config_name>/data", data = "<data>")] // TODO: Rename to /instance
async fn create_new_instance(
    config_name: &str,
    query: RawQuery,
    data: String,
    state: &State<StateManager>,
) {
    let service = state.get_service();

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

    service
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
    let service = state.get_service();

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

    service
        .save_config_instance(config_name, instance, labels, &data)
        .await
        .unwrap();
}

// #[put("/config/<config_name>")]
// async fn create_config(config_name: &str, state: &State<StateManager>) -> Status {
//     let adapter = state.get_adapter();
//     let result = adapter.create_config(config_name).await;

//     match result {
//         Ok(()) => Status::Ok,
//         Err(e) => match e {
//             CreateConfigError::StorageError { message } => {
//                 println!("Failed to create config {config_name}, error: {message}");
//                 Status::InternalServerError
//             }
//             CreateConfigError::DuplicateConfigError { name: _ } => Status::BadRequest,
//         },
//     }
// }

// #[get("/config/<config_name>/instance/<instance>/revisions")]
// async fn get_instance_revisions(
//     config_name: &str,
//     instance: &str,
//     state: &State<StateManager>,
// ) -> Option<Json<Vec<ConfigInstanceRevision>>> {
//     let adapter = state.get_adapter();

//     if let Some(data) = adapter.get_instance_revisions(config_name, instance).await {
//         return Some(Json(data));
//     }
//     return None;
// }

// #[post("/config/<config_name>/instance/<instance>/revision/<revision>/current")] // TODO: This should be renamed to /submit
// async fn update_instance_current_revision(
//     config_name: &str,
//     instance: &str,
//     revision: &str,
//     state: &State<StateManager>,
// ) {
//     let adapter = state.get_adapter();

//     adapter
//         .update_instance_current_revision(config_name, instance, revision)
//         .await
//         .unwrap();
// }

// #[post("/config/<config_name>/instance/<instance>/revision/<revision>/approve")]
// async fn approve_pending_instance_revision(
//     config_name: &str,
//     instance: &str,
//     revision: &str,
//     state: &State<StateManager>,
// ) {
//     let adapter = state.get_adapter();

//     adapter
//         .approve_pending_instance_revision(config_name, instance, revision)
//         .await
//         .unwrap();
// }

fn create_service() -> impl StorageService {
    let adapter_name = env::var("YAKMAN_ADAPTER").expect("$YAKMAN_ADAPTER is not set");

    // TODO: handle non file storage
    return match adapter_name.as_str() {
        // "REDIS" => Box::new(create_redis_adapter()),
        // "POSTGRES" => Box::new(create_postgres_adapter()),
        "LOCAL_FILE_SYSTEM" => {
            let adapter = Box::new(create_local_file_adapter());
            FileBasedStorageService { adapter: adapter }
        }
        _ => panic!("Unsupported adapter {adapter_name}"),
    };
}
