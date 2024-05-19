mod adapters;
mod api;
mod auth;
mod error;
mod middleware;
mod model;
mod notifications;
mod services;
mod settings;

use crate::adapters::init_adapter_from_env;
use crate::api::YakManApiDoc;
use crate::auth::oauth_service::YakManOAuthService;
use crate::middleware::roles::extract_roles;
use crate::middleware::YakManPrincipleTransformer;
use actix_middleware_etag::Etag;
use actix_web::middleware::Compress;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_grants::GrantsMiddleware;
use auth::oauth_service::{OAuthDisabledService, OAuthService};
use auth::token::YakManTokenService;
use dotenvy::dotenv;
use services::{kv_storage_service::KVStorageService, StorageService};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init();

    let adapter = init_adapter_from_env().await;
    let storage_service: Arc<dyn StorageService> = Arc::new(KVStorageService::new(adapter.clone()));

    if settings::is_snapshot_backups_enabled() {
        services::snapshot::register_snapshot_worker(adapter);
    } else {
        log::info!("Snapshot backups disabled");
    }

    storage_service
        .initialize_storage()
        .await
        .expect("Failed to initialize storage");

    let oauth_service = create_oauth_service(storage_service.clone()).await;
    let jwt_service = Arc::new(
        YakManTokenService::from_env()
            .map_err(|e| log::error!("{e}"))
            .expect("Failed to create jwt service"),
    );

    let openapi = YakManApiDoc::openapi();

    let x = 4;

    let (host, port) = yakman_host_port_from_env();
    log::info!("Launching YakMan Backend on {host}:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(storage_service.clone()))
            .app_data(web::Data::new(jwt_service.clone()))
            .app_data(web::Data::new(oauth_service.clone()))
            .wrap(Etag)
            .wrap(Compress::default())
            .wrap(Logger::new("%s %r"))
            .wrap(GrantsMiddleware::with_extractor(extract_roles))
            .wrap(YakManPrincipleTransformer)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .configure(api::register_routes)
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

async fn create_oauth_service(storage: Arc<dyn StorageService>) -> Arc<dyn OAuthService> {
    if settings::is_oauth_enabled() {
        let oauth_service = YakManOAuthService::new(storage.clone()).await.unwrap();
        return Arc::new(oauth_service);
    }
    return Arc::new(OAuthDisabledService::new());
}

/// Testing utilities and boilerplate setup code
#[cfg(test)]
mod test_utils {
    use crate::{
        adapters::{in_memory::InMemoryStorageAdapter, KVStorageAdapter},
        auth::{
            oauth_service::{MockOAuthService, OAuthService},
            token::{MockTokenService, TokenService},
        },
        services::{kv_storage_service::KVStorageService, StorageService},
    };
    use actix_web::{body::to_bytes, dev::ServiceResponse};
    use anyhow::{bail, Result};
    use serde_json::Value;
    use std::sync::Arc;

    pub fn prepare_for_actix_test() -> Result<()> {
        let _ = env_logger::try_init();

        Ok(())
    }

    pub async fn test_storage_service() -> Result<Arc<dyn StorageService>> {
        let adapter = InMemoryStorageAdapter::new();
        adapter.initialize_yakman_storage().await?;
        let service: KVStorageService = KVStorageService::new(Arc::new(adapter));
        return Ok(Arc::new(service));
    }

    #[allow(dead_code)]
    pub async fn test_oauth_service() -> Result<Arc<dyn OAuthService>> {
        return Ok(Arc::new(MockOAuthService::new()));
    }

    #[allow(dead_code)]
    pub async fn test_token_service() -> Result<Arc<dyn TokenService>> {
        return Ok(Arc::new(MockTokenService::new()));
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
        use actix_web_grants::authorities::AuthoritiesExtractor;
        use anyhow::Result;
        use std::{collections::HashSet, future::ready};

        pub struct FakeRoleExtractor {
            role_bindings: HashSet<YakManRoleBinding>,
        }

        impl FakeRoleExtractor {
            pub fn new(role_bindings: Vec<YakManRoleBinding>) -> FakeRoleExtractor {
                FakeRoleExtractor {
                    role_bindings: HashSet::from_iter(role_bindings.into_iter()),
                }
            }
        }

        impl<'a> AuthoritiesExtractor<'a, ServiceRequest, YakManRoleBinding> for FakeRoleExtractor {
            type Future = core::future::Ready<Result<HashSet<YakManRoleBinding>, Error>>;

            fn extract(&self, _request: &'a mut ServiceRequest) -> Self::Future {
                return ready(Ok(self.role_bindings.clone()));
            }
        }

        pub async fn admin_role(
            _req: &ServiceRequest,
        ) -> Result<HashSet<YakManRoleBinding>, Error> {
            let mut roles: HashSet<YakManRoleBinding> = HashSet::new();
            roles.insert(YakManRoleBinding::GlobalRoleBinding(YakManRole::Admin));
            return Ok(roles);
        }

        pub async fn approver_role(
            _req: &ServiceRequest,
        ) -> Result<HashSet<YakManRoleBinding>, Error> {
            let mut roles: HashSet<YakManRoleBinding> = HashSet::new();
            roles.insert(YakManRoleBinding::GlobalRoleBinding(YakManRole::Approver));
            return Ok(roles);
        }

        pub async fn operator_role(
            _req: &ServiceRequest,
        ) -> Result<HashSet<YakManRoleBinding>, Error> {
            let mut roles: HashSet<YakManRoleBinding> = HashSet::new();
            roles.insert(YakManRoleBinding::GlobalRoleBinding(YakManRole::Operator));
            return Ok(roles);
        }

        pub async fn viewer_role(
            _req: &ServiceRequest,
        ) -> Result<HashSet<YakManRoleBinding>, Error> {
            let mut roles: HashSet<YakManRoleBinding> = HashSet::new();
            roles.insert(YakManRoleBinding::GlobalRoleBinding(YakManRole::Viewer));
            return Ok(roles);
        }
    }
}
