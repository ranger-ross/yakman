mod adapters;
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

use crate::adapters::local_file_adapter::create_local_file_adapter;

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

    HttpServer::new(move || App::new().app_data(state.clone()).service(configs))
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
struct YakManError {
    error: String,
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

#[get("/configs")]
async fn configs(state: web::Data<StateManager>) -> actix_web::Result<impl Responder, YakManError> {
    let service = state.get_service();

    return match service.get_configs().await {
        Ok(data) => Ok(web::Json(data)),
        Err(err) => Err(YakManError::from(err)),
    };
}

// #[derive(Debug, Serialize)]
// struct Error {
//     msg: String,
//     status: u16,
// }

// impl Display for Error {
//   fn fmt(&self, f: &mut Formatter) -> FmtResult {
//     write!(f, "{}", to_string_pretty(self).unwrap())
//   }
// }

// impl ResponseError for Error {
//     // builds the actual response to send back when an error occurs
//     fn render_response(&self) -> HttpResponse {
//         let err_json = json!({ "error": self.msg });
//         HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json(err_json)
//     }

//     fn status_code(&self) -> StatusCode {
//         StatusCode::INTERNAL_SERVER_ERROR
//     }

//     fn error_response(&self) -> HttpResponse<BoxBody> {
//         let mut res = HttpResponse::new(self.status_code());

//         let mut buf = web::BytesMut::new();
//         let _ = write!(helpers::MutWriter(&mut buf), "{}", self);

//         let mime = mime::TEXT_PLAIN_UTF_8.try_into_value().unwrap();
//         res.headers_mut()
//             .insert(actix_web::http::header::CONTENT_TYPE, mime);

//         res.set_body(BoxBody::new(buf))
//     }

//     fn __private_get_type_id__(&self, _: PrivateHelper) -> (std::any::TypeId, PrivateHelper)
//     where
//         Self: 'static,
//     {
//         (std::any::TypeId::of::<Self>(), PrivateHelper(()))
//     }
// }

// fn index(_: HttpRequest) -> impl Future<Item = HttpResponse, Error = Error> {
//     Err(Error {
//         msg: "an example error message".to_string(),
//         status: 400,
//     })
// }

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

// #[get("/configs")]
// async fn configs(state: &State<StateManager>) -> GetConfigsResponse {
//     let service = state.get_service();

//     return match service.get_configs().await {
//         Ok(data) => GetConfigsResponse::ConfigData(Json(data)),
//         Err(err) => GetConfigsResponse::Error(Json(GenericError {
//             error: err.to_string(),
//         })),
//     };
// }

// #[get("/labels")]
// async fn labels(state: &State<StateManager>) -> Json<Vec<LabelType>> {
//     let service = state.get_service();
//     return Json(service.get_labels().await.unwrap());
// }

// #[put("/labels", data = "<data>")]
// async fn create_label(data: String, state: &State<StateManager>) -> Status {
//     let service = state.get_service();

//     let label_type: Option<LabelType> = serde_json::from_str(&data).ok();

//     if let Some(label_type) = label_type {
//         return match service.create_label(label_type).await {
//             Ok(()) => Status::Ok,
//             Err(e) => match e {
//                 CreateLabelError::DuplicateLabelError { name: _ } => Status::BadRequest,
//                 CreateLabelError::EmptyOptionsError => Status::BadRequest,
//                 CreateLabelError::InvalidPriorityError { prioity: _ } => Status::BadRequest,
//                 CreateLabelError::StorageError { message } => {
//                     println!("Failed to create label, error: {message}");
//                     Status::InternalServerError
//                 }
//             },
//         };
//     }

//     return Status::BadRequest; // Bad input so parse failed
// }

// #[get("/instances/<id>")]
// async fn get_instance_by_id(
//     id: &str,
//     state: &State<StateManager>,
// ) -> Option<Json<Vec<ConfigInstance>>> {
//     let service = state.get_service();
//     return match service.get_config_instance_metadata(id).await.unwrap() {
//         Some(data) => Some(Json(data)),
//         None => None,
//     };
// }

// // // TODO: Standardize REST endpoint naming

// #[get("/config/<config_name>/instance")]
// async fn data(config_name: &str, query: RawQuery, state: &State<StateManager>) -> Option<String> {
//     let service = state.get_service();

//     let labels: Vec<Label> = query
//         .params
//         .iter()
//         .map(|param| Label {
//             label_type: param.0.to_string(),
//             value: param.1.to_string(),
//         })
//         .collect();

//     println!("Search for config {config_name} with labels: {:?}", labels);

//     return service
//         .get_config_data_by_labels(config_name, labels)
//         .await
//         .unwrap();
// }

// #[get("/config/<config_name>/instance/<instance>")]
// async fn instance(
//     config_name: &str,
//     instance: &str,
//     state: &State<StateManager>,
// ) -> Option<String> {
//     let service = state.get_service();
//     return service
//         .get_config_data(config_name, instance)
//         .await
//         .unwrap();
// }

// #[put("/config/<config_name>/data", data = "<data>")] // TODO: Rename to /instance
// async fn create_new_instance(
//     config_name: &str,
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

//     // TODO: do validation
//     // - config exists
//     // - labels are valid
//     // - not a duplicate?

//     service
//         .create_config_instance(config_name, labels, &data)
//         .await
//         .unwrap();
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

// #[put("/config/<config_name>")]
// async fn create_config(config_name: &str, state: &State<StateManager>) -> Status {
//     let service = state.get_service();
//     let result = service.create_config(config_name).await;

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
