mod adapters;
mod config_man;
mod config_man_state;
mod data_types;
mod local_file_adapter;

use adapters::ConfigStorageAdapter;
use data_types::{AppConfig, AppConfigInstance, AppLabel, AppLabelType};
use local_file_adapter::LocalFileStorageAdapter;
use rocket::{
    request::{FromRequest, Outcome},
    serde::json::Json,
    Request,
};
use std::{collections::HashMap, vec};

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

#[get("/instances/<id>")] // TODO: add {id}
fn instances(id: &str) -> Option<Json<Vec<AppConfigInstance>>> {
    let ad = LocalFileStorageAdapter {
        path: "/home/ross/projects/config-manager/testing-directory".to_string(),
    };

    return match ad.get_config_instance_metadata(id) {
        Some(data) => Some(Json(data)),
        None => None,
    };
}

#[derive(Debug)]
struct RawQuery {
    params: HashMap<String, String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RawQuery {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let uri = request.uri().to_string();
        let mut m: HashMap<String, String> = HashMap::new();

        if uri.contains("?") {
            let index = uri.find("?").unwrap();
            let query: String = uri.chars().skip(index + 1).collect();

            for param in query.split("&") {
                let mut parts = param.split("=");
                let key = parts.next().unwrap();
                let value = parts.next().unwrap();
                m.insert(key.to_string(), value.to_string());
            }
        }

        return Outcome::Success(RawQuery { params: m });
    }
}

#[get("/data/<id>")]
fn data(id: &str, query: RawQuery) -> Option<String> {
    println!("{:?}", query.params);

    let ad = LocalFileStorageAdapter {
        path: "/home/ross/projects/config-manager/testing-directory".to_string(),
    };

    return ad.get_config_data(
        id,
        vec![AppLabel {
            // TODO: Make labels dynamic
            label_type_id: 300,
            value: "option 1".to_string(),
        }],
    );
}
