mod adapters;
mod api_routes;
mod services;
mod utils;

use adapters::{
    errors::{CreateConfigError, CreateLabelError},
    GenericStorageError,
};
// use rocket::{
//     http::Status,
//     serde::json::{serde_json, Json},
//     State,
// };
use serde::{Deserialize, Serialize};
use services::file_based_storage_service::{FileBasedStorageService, StorageService};
use std::{
    env,
    fmt::{Display, Formatter},
    future::Future,
    io::ErrorKind,
    rc::Rc,
    sync::{Arc, Mutex},
    vec,
};
use yak_man_core::{
    load_yak_man_settings,
    model::{Config, ConfigInstance, ConfigInstanceRevision, Label, LabelType},
};

use crate::{
    adapters::local_file_adapter::create_local_file_adapter,
    api_routes::{create_label, get_configs, get_data_by_labels, get_instance_by_id, get_labels, get_instance, create_new_instance, create_config},
};

use actix_web::{
    body::BoxBody,
    dev::Server,
    get,
    http::{header::ContentType, StatusCode},
    post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError,
};

#[derive(Clone)]
struct StateManager {
    service: Arc<dyn StorageService>,
}

impl StateManager {
    fn get_service(&self) -> &dyn StorageService {
        return self.service.as_ref();
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = load_yak_man_settings();
    println!("Settings: {:?}", settings);

    let service = create_service();

    service
        .initialize_storage()
        .await
        .expect("Failed to initialize storage");

    let state = web::Data::new(StateManager {
        service: Arc::new(service),
    });

    println!("Starting server");
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(get_configs)
            .service(get_labels)
            .service(create_label)
            .service(get_data_by_labels)
            .service(get_instance_by_id)
            .service(get_instance)
            .service(create_new_instance)
            .service(create_config)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await

    // rocket::build()
    //     .manage(StateManager {
    //         service: Box::new(service),
    //     })
    //     .mount(
    //         "/",
    //         routes![
    //             configs,
    //             labels,
    //             create_label,
    //             get_instance_by_id,
    //             data,
    //             create_new_instance,
    //             instance,
    //             create_config,
    //             update_new_instance,
    //             get_instance_revisions,
    //             update_instance_current_revision,
    //             approve_pending_instance_revision
    //         ],
    //     )
}

use actix_web::error;
use derive_more::{Display, Error};

#[derive(Debug, Display, Error, Serialize)]
pub struct YakManError {
    error: String,
}

impl YakManError {
    fn new(error: &str) -> YakManError {
        YakManError {
            error: String::from(error),
        }
    }
}

impl error::ResponseError for YakManError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(serde_json::to_string(self).unwrap_or(String::from("{}"))) // TODO: add internal server error message
    }
}

impl From<GenericStorageError> for YakManError {
    fn from(err: GenericStorageError) -> Self {
        YakManError {
            error: err.to_string(),
        }
    }
}

// #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
// struct GenericError {
//     error: String,
// }
// #[derive(Responder)]
// enum GetConfigsResponse {
//     #[response(status = 200, content_type = "json")]
//     ConfigData(Json<Vec<Config>>),
//     #[response(status = 500, content_type = "json")]
//     Error(Json<GenericError>),
// }

// #[post("/config/<config_name>/instance/<instance>", data = "<data>")]
// async fn update_new_instance(
//     config_name: &str,
//     instance: &str,
//     query: RawQuery,
//     data: String,
//     state: &State<StateManager>,
// ) {
//     let service = state.get_service();

//     let labels: Vec<Label> = query
//         .params
//         .iter()
//         .map(|param| Label {
//             label_type: param.0.to_string(),
//             value: param.1.to_string(),
//         })
//         .collect();

//     println!("lables {:?}", &labels);

//     // TODO: do validation
//     // - config exists
//     // - labels are valid
//     // - not a duplicate?

//     service
//         .save_config_instance(config_name, instance, labels, &data)
//         .await
//         .unwrap();
// }


// #[get("/config/<config_name>/instance/<instance>/revisions")]
// async fn get_instance_revisions(
//     config_name: &str,
//     instance: &str,
//     state: &State<StateManager>,
// ) -> Option<Json<Vec<ConfigInstanceRevision>>> {
//     let service = state.get_service();

//     if let Some(data) = service
//         .get_instance_revisions(config_name, instance)
//         .await
//         .unwrap()
//     {
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
//     let service = state.get_service();

//     service
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
//     let service = state.get_service();

//     service
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
