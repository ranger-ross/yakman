mod adapters;
mod api;
mod auth;
mod error;
mod middleware;
mod services;

extern crate dotenv;

use crate::auth::oauth_service::OauthService;
use crate::{
    adapters::local_file_adapter::create_local_file_adapter, middleware::roles::extract_roles,
};
use actix_middleware_etag::Etag;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_grants::GrantsMiddleware;
use adapters::redis_adapter::create_redis_adapter;
use auth::token::TokenService;
use dotenv::dotenv;
use log::info;
use services::{kv_storage_service::KVStorageService, StorageService};
use std::{env, sync::Arc};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use yak_man_core::model::response::GetUserRolesResponse;
use yak_man_core::{
    load_yak_man_settings,
    model::{
        request::{CreateConfigPayload, CreateProjectPayload},
        Config, ConfigInstance, ConfigInstanceChange, ConfigInstanceRevision, Label, LabelType,
        YakManProject, YakManRole, YakManSettings, YakManUser,
    },
};

#[derive(Clone)]
pub struct StateManager {
    service: Arc<dyn StorageService>,
    oauth_service: Arc<OauthService>,
    jwt_service: Arc<TokenService>,
}

impl StateManager {
    fn get_service(&self) -> &dyn StorageService {
        return self.service.as_ref();
    }
    fn get_oauth_service(&self) -> &OauthService {
        return self.oauth_service.as_ref();
    }
    fn get_token_service(&self) -> &TokenService {
        return self.jwt_service.as_ref();
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        api::oauth::oauth_init,
        api::oauth::oauth_exchange,
        api::oauth::oauth_refresh,
        api::oauth::get_user_roles,
        api::projects::get_projects,
        api::projects::create_project,
        api::configs::get_configs,
        api::configs::create_config,
        api::labels::get_labels,
        api::labels::create_label,
        api::instances::get_instances_by_config_name,
        api::instances::get_instance,
        api::instances::create_new_instance,
        api::instances::update_new_instance,
        api::data::get_instance_data,
        api::data::get_revision_data,
        api::revisions::get_instance_revisions,
        api::revisions::submit_instance_revision,
        api::revisions::approve_pending_instance_revision,
    ),
    components(
        schemas(
            Config, LabelType, Label, ConfigInstance, ConfigInstanceRevision, ConfigInstanceChange, YakManSettings, 
            YakManProject, YakManRole, YakManUser, CreateConfigPayload, CreateProjectPayload, GetUserRolesResponse
        )
    ),
    tags(
        (name = "api::oauth", description = "OAuth endpoints"),
        (name = "api::projects", description = "Project management endpoints"),
        (name = "api::configs", description = "Config management endpoints"),
        (name = "api::labels", description = "Label management endpoints"),
        (name = "api::instances", description = "Config Instance management endpoints"),
        (name = "api::data", description = "Config data fetching endpoints"),
        (name = "api::revisions", description = "Config Instance Revision management endpoints"),
    )
)]
struct ApiDoc;

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
    let jwt_service = TokenService::from_env().expect("Failed to create jwt service");

    let state = web::Data::new(StateManager {
        service: arc,
        oauth_service: Arc::new(oauth_service),
        jwt_service: Arc::new(jwt_service),
    });

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Etag::default())
            .wrap(Logger::new("%s %r"))
            .wrap(GrantsMiddleware::with_extractor(extract_roles))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            // OAuth
            .service(api::oauth::oauth_init)
            .service(api::oauth::oauth_exchange)
            .service(api::oauth::oauth_refresh)
            .service(api::oauth::get_user_roles)
            // Projects
            .service(api::projects::get_projects)
            .service(api::projects::create_project)
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
            .service(api::data::get_instance_data)
            .service(api::data::get_revision_data)
            // Revisions
            .service(api::revisions::get_instance_revisions)
            .service(api::revisions::submit_instance_revision)
            .service(api::revisions::approve_pending_instance_revision)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}

fn create_service() -> impl StorageService {
    let adapter_name = env::var("YAKMAN_ADAPTER").expect("$YAKMAN_ADAPTER is not set");

    // TODO: handle non file storage
    return match adapter_name.as_str() {
        "REDIS" => {
            let adapter = Box::new(create_redis_adapter());
            KVStorageService { adapter: adapter }
        },
        // "POSTGRES" => Box::new(create_postgres_adapter()),
        "LOCAL_FILE_SYSTEM" => {
            let adapter = Box::new(create_local_file_adapter());
            KVStorageService { adapter: adapter }
        }
        _ => panic!("Unsupported adapter {adapter_name}"),
    };
}
