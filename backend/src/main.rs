mod adapters;
mod api;
mod services;

use adapters::errors::GenericStorageError;

use serde::Serialize;
use services::{file_based_storage_service::FileBasedStorageService, StorageService};
use std::{env, sync::Arc};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use yak_man_core::{
    load_yak_man_settings,
    model::{Config, ConfigInstance, ConfigInstanceRevision, Label, LabelType, YakManSettings},
};

use crate::{
    adapters::local_file_adapter::create_local_file_adapter,
};

use actix_web::{http::header::ContentType, web, App, HttpResponse, HttpServer};

#[derive(Clone)]
pub struct StateManager {
    service: Arc<dyn StorageService>,
}

impl StateManager {
    fn get_service(&self) -> &dyn StorageService {
        return self.service.as_ref();
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        api::configs::get_configs,
        api::configs::create_config,
        api::labels::get_labels,
        api::labels::create_label,
        api::instances::get_data_by_labels,
        api::instances::get_instances_by_config_name,
        api::instances::get_instance,
        api::instances::create_new_instance,
        api::instances::update_new_instance,
        api::revisions::get_instance_revisions,
        api::revisions::submit_instance_revision,
        api::revisions::approve_pending_instance_revision,
    ),
    components(
        schemas(Config, LabelType, Label, ConfigInstance, ConfigInstanceRevision, YakManSettings)
    ),
    tags(
        (name = "api::configs", description = "Config management endpoints"),
        (name = "api::labels", description = "Label management endpoints"),
        (name = "api::instances", description = "Config Instance management endpoints"),
        (name = "api::revisions", description = "Config Instance Revision management endpoints"),
    )
)]
struct ApiDoc;

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

    let openapi = ApiDoc::openapi();

    println!("Starting server");
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            // Configs
            .service(api::configs::get_configs)
            .service(api::configs::create_config)
            // Labels
            .service(api::labels::get_labels)
            .service(api::labels::create_label)
            // Instances
            .service(api::instances::get_data_by_labels)
            .service(api::instances::get_instances_by_config_name)
            .service(api::instances::get_instance)
            .service(api::instances::create_new_instance)
            .service(api::instances::update_new_instance)
            // Revisions
            .service(api::revisions::get_instance_revisions)
            .service(api::revisions::submit_instance_revision)
            .service(api::revisions::approve_pending_instance_revision)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
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
