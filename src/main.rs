mod adapters;
mod config_man;
mod data_types;

use adapters::{ConfigStorageAdapter, LocalFileStorageAdapter};
use data_types::AppConfig;
use rocket::serde::json::Json;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    let settings = config_man::load_config_man_settings();
    println!("Settings: {:?}", settings);

    rocket::build().mount("/", routes![configs])
}

#[get("/configs")]
fn configs() -> Json<Vec<AppConfig>> {
    let ad = LocalFileStorageAdapter {
        path: "/home/ross/projects/config-manager/testing-directory".to_string(),
    };

    return Json(ad.load_configs());
}
