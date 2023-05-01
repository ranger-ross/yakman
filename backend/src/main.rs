mod adapters;
mod api;
mod services;

extern crate dotenv;

use crate::adapters::local_file_adapter::create_local_file_adapter;
use crate::services::oauth_service::OauthService;
use actix_middleware_etag::Etag;
use actix_web::{
    dev::ServiceRequest, http::header::ContentType, middleware::Logger, web, App, Error,
    HttpResponse, HttpServer,
};
use actix_web_grants::GrantsMiddleware;
use adapters::errors::GenericStorageError;
use dotenv::dotenv;
use log::info;
use serde::Serialize;
use services::{file_based_storage_service::FileBasedStorageService, StorageService};
use std::{env, sync::Arc};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use yak_man_core::{
    load_yak_man_settings,
    model::{
        Config, ConfigInstance, ConfigInstanceChange, ConfigInstanceRevision, Label, LabelType,
        YakManRole, YakManSettings,
    },
};

#[derive(Clone)]
pub struct StateManager {
    service: Arc<dyn StorageService>,
    oauth_service: Arc<OauthService>,
}

impl StateManager {
    fn get_service(&self) -> &dyn StorageService {
        return self.service.as_ref();
    }
    fn get_oauth_service(&self) -> &OauthService {
        return self.oauth_service.as_ref();
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        api::configs::get_configs,
        api::configs::create_config,
        api::labels::get_labels,
        api::labels::create_label,
        api::instances::get_instances_by_config_name,
        api::instances::get_instance,
        api::instances::create_new_instance,
        api::instances::update_new_instance,
        api::data::get_data_by_labels,
        api::data::get_instance_data,
        api::revisions::get_instance_revisions,
        api::revisions::submit_instance_revision,
        api::revisions::approve_pending_instance_revision,
    ),
    components(
        schemas(Config, LabelType, Label, ConfigInstance, ConfigInstanceRevision, ConfigInstanceChange, YakManSettings)
    ),
    tags(
        (name = "api::configs", description = "Config management endpoints"),
        (name = "api::labels", description = "Label management endpoints"),
        (name = "api::instances", description = "Config Instance management endpoints"),
        (name = "api::data", description = "Config data fetching endpoints"),
        (name = "api::revisions", description = "Config Instance Revision management endpoints"),
    )
)]
struct ApiDoc;

// You can use custom type instead of String
async fn extract(req: &ServiceRequest) -> Result<Vec<YakManRole>, Error> {
    let state = req.app_data::<web::Data<StateManager>>().unwrap();
    let cookie_name = "access_token";
    let cookies = req.cookies().unwrap();
    let token = cookies.iter().find(|c| c.name() == cookie_name);

    if token.is_none() {
        return Ok(vec![]);
    }

    let username = state.get_oauth_service().get_username(token.unwrap().value()).await.unwrap();

    info!("user: {}", username);

    let user = state
        .get_service()
        .get_user(&username)
        .await
        .unwrap()
        .unwrap();

    Ok(vec![user.role])
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init();

    let settings = load_yak_man_settings();
    info!("Settings {settings:?}");

    let service = create_service();

    service
        .initialize_storage()
        .await
        .expect("Failed to initialize storage");

    let arc = Arc::new(service);

    let oauth_service = OauthService::new(arc.clone());

    let state = web::Data::new(StateManager {
        service: arc,
        oauth_service: Arc::new(oauth_service),
    });

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Etag::default())
            .wrap(Logger::new("%s %r"))
            .wrap(GrantsMiddleware::with_extractor(extract))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(api::oauth::oauth_init)
            .service(api::oauth::oauth_exchange)
            // Admin
            .service(api::admin::get_yakman_users)
            .service(api::admin::create_yakman_user)
            // Configs
            .service(api::configs::get_configs)
            .service(api::configs::create_config)
            // Labels
            .service(api::labels::get_labels)
            .service(api::labels::create_label)
            // Instances
            .service(api::instances::get_instances_by_config_name)
            .service(api::instances::get_instance)
            .service(api::instances::create_new_instance)
            .service(api::instances::update_new_instance)
            // Data
            .service(api::data::get_data_by_labels)
            .service(api::data::get_instance_data)
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
