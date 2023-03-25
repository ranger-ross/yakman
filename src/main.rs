mod adapters;
mod config_man;
mod config_man_state;
mod data_types;

use std::vec;

use adapters::{ConfigStorageAdapter, LocalFileStorageAdapter};
use data_types::{AppConfig, AppConfigInstance, AppLabel, AppLabelType};
use rocket::serde::json::Json;

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
    let ad = LocalFileStorageAdapter {
        path: "/home/ross/projects/config-manager/testing-directory".to_string(),
    };

    return Json(ad.get_configs());
}

#[get("/labels")]
fn labels() -> Json<Vec<AppLabelType>> {
    let ad = LocalFileStorageAdapter {
        path: "/home/ross/projects/config-manager/testing-directory".to_string(),
    };

    return Json(ad.get_labels());
}

#[get("/instances")] // TODO: add {id}
fn instances() -> Json<Vec<AppConfigInstance>> {
    let ad = LocalFileStorageAdapter {
        path: "/home/ross/projects/config-manager/testing-directory".to_string(),
    };

    return Json(ad.get_config_instance_metadata(100));
}

#[get("/data")] // TODO: add {id} / {tags}
fn data() -> String {
    let ad = LocalFileStorageAdapter {
        path: "/home/ross/projects/config-manager/testing-directory".to_string(),
    };

    return ad.get_config_data(
        100,
        vec![AppLabel {
            label_type_id: 300,
            value: "option 1".to_string(),
        }],
    );
}
