mod adapters;
mod api;
mod auth;
mod error;
mod middleware;
mod model;
mod services;
mod settings;

extern crate dotenv;

use crate::api::YakManApiDoc;
use crate::auth::oauth_service::YakManOAuthService;
use crate::middleware::roles::extract_roles;
use crate::middleware::YakManPrincipleTransformer;
use crate::services::snapshot::SnapshotService;
use actix_middleware_etag::Etag;
use actix_web::middleware::Compress;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_grants::GrantsMiddleware;
use adapters::in_memory::InMemoryStorageAdapter;
use adapters::local_file::LocalFileStorageAdapter;
use adapters::redis::redis_adapter::RedisStorageAdapter;
use adapters::KVStorageAdapter;
use adapters::{
    aws_s3::AwsS3StorageAdapter,
    google_cloud_storage::google_cloud_storage_adapter::GoogleCloudStorageAdapter,
};
use anyhow::Context;
use auth::oauth_service::{OAuthDisabledService, OAuthService};
use auth::token::YakManTokenService;
use dotenv::dotenv;
use services::{kv_storage_service::KVStorageService, StorageService};
use std::time::Duration;
use std::{env, sync::Arc};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init();

    let adapter = create_adapter().await;
    let storage_service: Arc<dyn StorageService> = Arc::new(KVStorageService::new(adapter.clone()));

    //TODO: allow disabling
    tokio::spawn(async {
        let snapshot_service = SnapshotService::new(adapter);

        loop {
            snapshot_service.take_snapshot().await;

            // TODO: make dynamic and probably CRON
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

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

    let (host, port) = yakman_host_port_from_env();
    log::info!("Launching YakMan Backend on {host}:{port}");

    HttpServer::new(move || {
        let app = App::new()
            .app_data(web::Data::new(storage_service.clone()))
            .app_data(web::Data::new(jwt_service.clone()))
            .app_data(web::Data::new(oauth_service.clone()))
            .wrap(Etag::default())
            .wrap(Compress::default())
            .wrap(Logger::new("%s %r"))
            .wrap(GrantsMiddleware::with_extractor(extract_roles))
            .wrap(YakManPrincipleTransformer)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            );

        return api::register_routes(app);
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

async fn create_adapter() -> Arc<dyn KVStorageAdapter> {
    let adapter_name = env::var("YAKMAN_ADAPTER").expect("$YAKMAN_ADAPTER is not set");

    return match adapter_name.as_str() {
        "REDIS" => Arc::new(
            RedisStorageAdapter::from_env()
                .await
                .context("Failed to initialize Redis adapter")
                .unwrap(),
        ),
        "LOCAL_FILE_SYSTEM" => Arc::new(LocalFileStorageAdapter::from_env().await),
        "S3" => Arc::new(AwsS3StorageAdapter::from_env().await),
        "GOOGLE_CLOUD_STORAGE" => Arc::new(
            GoogleCloudStorageAdapter::from_env()
                .await
                .context("Failed to initialize Google Cloud Storage adapter")
                .unwrap(),
        ),
        "IN_MEMORY" => Arc::new(InMemoryStorageAdapter::new()),
        _ => panic!("Unsupported adapter {adapter_name}"),
    };
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
        let service: KVStorageService = KVStorageService::new(Box::new(adapter));
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
