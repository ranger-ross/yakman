mod adapters;
mod data_types;

use std::path::Path;

use adapters::{ConfigStorageAdapter, LocalFileStorageAdapter};
use data_types::{AppConfig, AppConfigInstance};
use rocket::serde::json::Json;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}

#[get("/config")]
fn index() -> Json<Vec<AppConfig>> {
    let ad = LocalFileStorageAdapter {
        path: "/home/ross/projects/config-manager/testing-directory".to_string(),
    };

    return Json(ad.load_configs());
}
