mod adapters;
mod api;
mod auth;
mod error;
mod middleware;
mod model;
mod services;

extern crate dotenv;

use crate::auth::oauth_service::YakManOauthService;
use crate::middleware::roles::extract_roles;
use crate::middleware::YakManPrincipleTransformer;
use actix_middleware_etag::Etag;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_grants::GrantsMiddleware;
use adapters::in_memory::InMemoryStorageAdapter;
use adapters::local_file::LocalFileStorageAdapter;
use adapters::redis::redis_adapter::RedisStorageAdapter;
use adapters::{
    aws_s3::AwsS3StorageAdapter,
    google_cloud_storage::google_cloud_storage_adapter::GoogleCloudStorageAdapter,
};
use anyhow::Context;
use api::oauth::{GetUserInfoResponse, OAuthExchangePayload, OAuthInitPayload, OAuthInitResponse};
use auth::oauth_service::OauthService;
use auth::token::{TokenService, YakManTokenService};
use dotenv::dotenv;
use log::info;
use model::response::RevisionPayload;
use model::{
    request::{CreateConfigPayload, CreateProjectPayload},
    ConfigInstance, ConfigInstanceChange, ConfigInstanceRevision, LabelType, YakManConfig,
    YakManLabel, YakManProject, YakManRole, YakManSettings, YakManUser,
};
use services::{kv_storage_service::KVStorageService, StorageService};
use std::{env, sync::Arc};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct StateManager {
    service: Arc<dyn StorageService>,
    oauth_service: Arc<dyn OauthService>,
    jwt_service: Arc<dyn TokenService>,
}

impl StateManager {
    fn get_service(&self) -> &dyn StorageService {
        return self.service.as_ref();
    }
    fn get_oauth_service(&self) -> &dyn OauthService {
        return self.oauth_service.as_ref();
    }
    fn get_token_service(&self) -> &dyn TokenService {
        return self.jwt_service.as_ref();
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        api::oauth::oauth_init,
        api::oauth::oauth_exchange,
        api::oauth::oauth_refresh,
        api::oauth::get_user_info,
        api::projects::get_projects,
        api::projects::create_project,
        api::configs::get_configs,
        api::configs::create_config,
        api::configs::delete_config,
        api::labels::get_labels,
        api::labels::create_label,
        api::instances::get_instances_by_config_name,
        api::instances::get_instance,
        api::instances::create_new_instance,
        api::instances::update_new_instance,
        api::data::get_instance_data,
        api::data::get_revision_data,
        api::revisions::get_instance_revisions,
        api::revisions::review_pending_instance_revision,
        api::revisions::apply_instance_revision,
        api::revisions::rollback_instance_revision,
    ),
    components(
        schemas(
            YakManConfig, LabelType, YakManLabel, ConfigInstance, ConfigInstanceRevision, ConfigInstanceChange, YakManSettings,
            YakManProject, YakManRole, YakManUser, CreateConfigPayload, CreateProjectPayload, GetUserInfoResponse,
            OAuthInitPayload, OAuthExchangePayload, OAuthInitResponse, RevisionPayload
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

    let service = create_service().await;

    service
        .initialize_storage()
        .await
        .expect("Failed to initialize storage");

    let arc = Arc::new(service);

    let oauth_service = YakManOauthService::new(arc.clone()).await.unwrap();
    let jwt_service = YakManTokenService::from_env()
        .map_err(|e| log::error!("{e}"))
        .expect("Failed to create jwt service");

    let state = web::Data::new(StateManager {
        service: arc,
        oauth_service: Arc::new(oauth_service),
        jwt_service: Arc::new(jwt_service),
    });

    let openapi = ApiDoc::openapi();

    let (host, port) = yakman_host_port_from_env();
    info!("Launching YakMan Backend on {host}:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(YakManPrincipleTransformer)
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
            .service(api::oauth::get_user_info)
            // Projects
            .service(api::projects::get_projects)
            .service(api::projects::create_project)
            // Admin
            .service(api::admin::get_yakman_users)
            .service(api::admin::create_yakman_user)
            .service(api::admin::get_api_keys)
            // Configs
            .service(api::configs::get_configs)
            .service(api::configs::create_config)
            .service(api::configs::delete_config)
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
            .service(api::revisions::review_pending_instance_revision)
            .service(api::revisions::apply_instance_revision)
            .service(api::revisions::rollback_instance_revision)
    })
    .bind((host, port))?
    .run()
    .await
}

fn yakman_host_port_from_env() -> (String, u16) {
    let host = std::env::var("YAKMAN_HOST").unwrap_or("0.0.0.0".to_string());
    let port = std::env::var("YAKMAN_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(8000);
    (host, port)
}

async fn create_service() -> impl StorageService {
    let adapter_name = env::var("YAKMAN_ADAPTER").expect("$YAKMAN_ADAPTER is not set");

    return match adapter_name.as_str() {
        "REDIS" => {
            let adapter = Box::new(
                RedisStorageAdapter::from_env()
                    .await
                    .context("Failed to initialize Redis adapter")
                    .unwrap(),
            );
            KVStorageService { adapter: adapter }
        }
        "LOCAL_FILE_SYSTEM" => {
            let adapter = Box::new(LocalFileStorageAdapter::from_env().await);
            KVStorageService { adapter: adapter }
        }
        "S3" => {
            let adapter = Box::new(AwsS3StorageAdapter::from_env().await);
            KVStorageService { adapter: adapter }
        }
        "GOOGLE_CLOUD_STORAGE" => {
            let adapter = Box::new(
                GoogleCloudStorageAdapter::from_env()
                    .await
                    .context("Failed to initialize Google Cloud Storage adapter")
                    .unwrap(),
            );
            KVStorageService { adapter: adapter }
        }
        "IN_MEMORY" => {
            let adapter = Box::new(InMemoryStorageAdapter::new());
            KVStorageService { adapter: adapter }
        }
        _ => panic!("Unsupported adapter {adapter_name}"),
    };
}

fn load_yak_man_settings() -> YakManSettings {
    return YakManSettings {
        version: "0.0.1".to_string(),
    };
}

/// Testing utilities and boilerplate setup code
#[cfg(test)]
mod test_utils {
    use crate::{
        adapters::{in_memory::InMemoryStorageAdapter, KVStorageAdapter},
        auth::{oauth_service::MockOauthService, token::MockTokenService},
        services::kv_storage_service::KVStorageService,
        StateManager,
    };
    use actix_web::{body::to_bytes, dev::ServiceResponse};
    use anyhow::{bail, Result};
    use serde_json::Value;
    use std::sync::Arc;

    pub fn prepare_for_actix_test() -> Result<()> {
        let _ = env_logger::try_init();

        Ok(())
    }

    pub async fn test_state_manager() -> Result<StateManager> {
        let adapter = InMemoryStorageAdapter::new();
        adapter.initialize_yakman_storage().await?;
        let service: KVStorageService = KVStorageService {
            adapter: Box::new(adapter),
        };

        let token_service = MockTokenService::new();
        let oauth_service = MockOauthService::new();

        return Ok(StateManager {
            jwt_service: Arc::new(token_service),
            oauth_service: Arc::new(oauth_service),
            service: Arc::new(service),
        });
    }

    pub async fn body_to_json_value(res: ServiceResponse) -> Result<Value> {
        let body = res.into_body();
        let bytes = match to_bytes(body).await {
            Ok(b) => b,
            Err(_) => bail!("Failed to extract response data as bytes"),
        };

        let value_as_string = String::from_utf8(bytes.to_vec())?;

        Ok(serde_json::from_str(&value_as_string)?)
    }

    /// Utility for stubbing roles extractor in tests
    pub mod fake_roles {
        #![allow(dead_code)]

        use crate::{middleware::roles::YakManRoleBinding, model::YakManRole};
        use actix_web::{dev::ServiceRequest, Error};
        use actix_web_grants::permissions::PermissionsExtractor;
        use anyhow::Result;
        use std::future::ready;

        pub struct FakeRoleExtractor {
            role_bindings: Vec<YakManRoleBinding>,
        }

        impl FakeRoleExtractor {
            pub fn new(role_bindings: Vec<YakManRoleBinding>) -> FakeRoleExtractor {
                FakeRoleExtractor {
                    role_bindings: role_bindings,
                }
            }
        }

        impl<'a> PermissionsExtractor<'a, ServiceRequest, YakManRoleBinding> for FakeRoleExtractor {
            type Future = core::future::Ready<Result<Vec<YakManRoleBinding>, Error>>;

            fn extract(&self, _request: &'a mut ServiceRequest) -> Self::Future {
                return ready(Ok(self.role_bindings.clone()));
            }
        }

        pub async fn admin_role(_req: &ServiceRequest) -> Result<Vec<YakManRoleBinding>, Error> {
            return Ok(vec![YakManRoleBinding::GlobalRoleBinding(
                YakManRole::Admin,
            )]);
        }

        pub async fn approver_role(_req: &ServiceRequest) -> Result<Vec<YakManRoleBinding>, Error> {
            return Ok(vec![YakManRoleBinding::GlobalRoleBinding(
                YakManRole::Approver,
            )]);
        }

        pub async fn operator_role(_req: &ServiceRequest) -> Result<Vec<YakManRoleBinding>, Error> {
            return Ok(vec![YakManRoleBinding::GlobalRoleBinding(
                YakManRole::Operator,
            )]);
        }

        pub async fn viewer_role(_req: &ServiceRequest) -> Result<Vec<YakManRoleBinding>, Error> {
            return Ok(vec![YakManRoleBinding::GlobalRoleBinding(
                YakManRole::Viewer,
            )]);
        }
    }
}
