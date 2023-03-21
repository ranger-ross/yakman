use rocket::http::Status;
use rocket::response::{content, status};
use rocket::serde::json::Json;
use serde::Serialize;

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

#[derive(Serialize)]
struct AppConfig {
    id: i32,
    name: String,
}

#[derive(Serialize)]
struct AppLabelType {
    id: i32,
    name: String,
}

#[derive(Serialize)]
struct AppLabel {
    label_type: AppLabelType,
    value: String, // TODO: more powerful generics?
}

#[derive(Serialize)]
struct AppConfigInstance {
    config: AppConfig,
    content: String,
    labels: Vec<AppLabel>,
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
