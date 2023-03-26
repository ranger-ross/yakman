mod adapters;
mod config_man;
mod config_man_state;
mod data_types;
mod local_file_adapter;
mod raw_query;
mod redis_adapter;

use adapters::ConfigStorageAdapter;
use data_types::{AppConfig, AppConfigInstance, AppLabel, AppLabelType};
use local_file_adapter::LocalFileStorageAdapter;
use redis_adapter::RedisStorageAdapter;
use rocket::serde::json::Json;
use std::vec;

use raw_query::RawQuery;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    let settings = config_man::load_config_man_settings();
    println!("Settings: {:?}", settings);

    rocket::build().mount("/", routes![configs, labels, instances, data])
}

#[get("/configs")]
fn configs() -> Json<Vec<AppConfig>> {
    let adapter = get_adapter();
    return Json(adapter.get_configs());
}

#[get("/labels")]
fn labels() -> Json<Vec<AppLabelType>> {
    let adapter = get_adapter();
    return Json(adapter.get_labels());
}

#[get("/instances/<id>")] // TODO: add {id}
fn instances(id: &str) -> Option<Json<Vec<AppConfigInstance>>> {
    let adapter = get_adapter();
    return match adapter.get_config_instance_metadata(id) {
        Some(data) => Some(Json(data)),
        None => None,
    };
}

#[get("/data/<id>")]
fn data(id: &str, query: RawQuery) -> Option<String> {
    let adapter = get_adapter();

    let labels: Vec<AppLabel> = query
        .params
        .iter()
        .map(|param| AppLabel {
            label_type: param.0.to_string(),
            value: param.1.to_string(),
        })
        .collect();

    println!("Search for config {} with labels: {:?}", id, labels);

    return adapter.get_config_data(id, labels);
}

// fn get_adapter() -> impl ConfigStorageAdapter {
//     return LocalFileStorageAdapter {
//         path: "/home/ross/projects/config-manager/testing-directory".to_string(),
//     };
// }

fn get_adapter() -> impl ConfigStorageAdapter {
    return RedisStorageAdapter {
        host: "127.0.0.1".to_string(),
        port: 6379,
        username: "".to_string(),
        password: "".to_string(),
    };
}
