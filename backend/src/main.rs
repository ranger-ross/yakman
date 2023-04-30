mod adapters;
mod api;
mod services;

use crate::adapters::local_file_adapter::create_local_file_adapter;
use crate::services::oauth_service::OauthService;
use actix_middleware_etag::Etag;
use actix_web::{
    http::header::ContentType, middleware::Logger, web, App, HttpResponse, HttpServer,
};
use adapters::errors::GenericStorageError;
use log::info;
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{
    reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Serialize;
use services::{file_based_storage_service::FileBasedStorageService, StorageService};
use std::{env, sync::Arc};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use yak_man_core::{
    load_yak_man_settings,
    model::{
        Config, ConfigInstance, ConfigInstanceChange, ConfigInstanceRevision, Label, LabelType,
        YakManSettings,
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // let auth_url: AuthUrl =
    //     AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
    //         .expect("Invalid authorization endpoint URL");
    // let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
    //     .expect("Invalid token endpoint URL");

    // // Create an OAuth2 client by specifying the client ID, client secret, authorization URL and
    // // token URL.
    // let client = BasicClient::new(
    //     ClientId::new(
    //         "797682569427-sta5hoe91q4com9ojps3vo3qs83fbliu.apps.googleusercontent.com".to_string(),
    //     ),
    //     Some(ClientSecret::new(
    //         "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".to_string(),
    //     )),
    //     auth_url,
    //     Some(token_url),
    // )
    // // Set the URL the user will be redirected to after the authorization process.
    // .set_redirect_uri(RedirectUrl::new("http://127.0.0.1:8080".to_string()).unwrap());

    // // Generate a PKCE challenge.
    // let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // // Generate the full authorization URL.
    // let (auth_url, csrf_token) = client
    //     .authorize_url(CsrfToken::new_random)
    //     // Set the desired scopes.
    //     .add_scope(Scope::new("email".to_string()))
    //     // .add_scope(Scope::new("offline_access".to_string()))
    //     // Set the PKCE code challenge.
    //     .set_pkce_challenge(pkce_challenge)
    //     .url();

    // // This is the URL you should redirect the user to, in order to trigger the authorization
    // // process.
    // println!("Browse to: {}", auth_url);

    // // Once the user has been redirected to the redirect URL, you'll have access to the
    // // authorization code. For security reasons, your code should verify that the `state`
    // // parameter returned by the server matches `csrf_state`.

    // // Now you can trade it for an access token.
    // let token_result = client
    //     .exchange_code(AuthorizationCode::new(
    //         "4/0AbUR2VO8CHeTH2i6oV9QDtdpwQQDdnJIe_ZLLQs0k6yGmlWPXzdVudW2C4ynYAhNUfCqQQ".to_string(),
    //     ))
    //     // Set the PKCE code verifier.
    //     .set_pkce_verifier(pkce_verifier)
    //     .request_async(async_http_client)
    //     .await
    //     .unwrap();

    let settings = load_yak_man_settings();
    info!("Settings {settings:?}");

    let service = create_service();

    service
        .initialize_storage()
        .await
        .expect("Failed to initialize storage");

    let oauth_service = OauthService {};

    let state = web::Data::new(StateManager {
        service: Arc::new(service),
        oauth_service: Arc::new(oauth_service),
    });

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Etag::default())
            .wrap(Logger::new("%s %r"))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(api::oauth::oauth_init)
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
