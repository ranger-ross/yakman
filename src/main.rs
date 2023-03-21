mod data_types;

use data_types::{AppConfig, AppConfigInstance};
use rocket::serde::json::Json;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}

#[get("/config")]
fn index() -> Json<AppConfig> {
    return Json(AppConfig {
        id: 123,
        name: "test".to_string(),
    });
}

fn load_config() -> AppConfigInstance {
    return AppConfigInstance {
        config: AppConfig {
            id: 100,
            name: "FirstConfig".to_string(),
        },
        content: "this is my config data".to_string(),
        labels: vec![],
    };
}

// The base storage adapter to be able to load config from external storage
trait ConfigStorageAdapter {
    fn load_configs() -> Vec<AppConfig>;

    fn load_config(id: i32) -> AppConfigInstance;
}
