use crate::{
    api::is_alphanumeric_kebab_case, services::errors::CreateConfigError, StateManager, YakManError,
};

use actix_web::{
    get, put,
    web::{self, Json},
    HttpResponse, Responder,
};
use actix_web_grants::proc_macro::has_any_role;
use log::{error, info};
use serde::Deserialize;
use yak_man_core::model::{request::CreateConfigPayload, YakManRole};

#[derive(Deserialize)]
pub struct GetConfigsQuery {
    pub project: Option<String>,
}

/// List of all configs
#[utoipa::path(responses((status = 200, body = Vec<Config>)))]
#[get("/configs")]
#[has_any_role(
    "YakManRole::Admin",
    "YakManRole::Approver",
    "YakManRole::Operator",
    "YakManRole::Viewer",
    type = "YakManRole"
)]
pub async fn get_configs(
    query: web::Query<GetConfigsQuery>,
    state: web::Data<StateManager>,
) -> actix_web::Result<impl Responder, YakManError> {
    let project = query.project.to_owned();
    info!("PROJECT: {project:?}");
    let service = state.get_service();
    return match service.get_configs(project).await {
        Ok(data) => Ok(web::Json(data)),
        Err(err) => Err(YakManError::from(err)),
    };
}

/// Create a new config
#[utoipa::path(request_body = CreateConfigPayload, responses((status = 200, body = String)))]
#[put("/configs")]
#[has_any_role("YakManRole::Admin", "YakManRole::Approver", type = "YakManRole")]
async fn create_config(
    payload: web::Json<CreateConfigPayload>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let payload = payload.into_inner();
    let config_name = payload.config_name.to_lowercase();
    let project_uuid = payload.project_uuid;

    if !is_alphanumeric_kebab_case(&config_name) {
        return HttpResponse::BadRequest()
            .body("Invalid config name. Must be alphanumeric kebab case");
    }

    let service = state.get_service();
    let result: Result<(), CreateConfigError> =
        service.create_config(&config_name, &project_uuid).await;

    return match result {
        Ok(()) => HttpResponse::Ok().body(""),
        Err(e) => match e {
            CreateConfigError::StorageError { message } => {
                error!("Failed to create config {config_name}, error: {message}");
                HttpResponse::InternalServerError().body("Failed to create config")
            }
            CreateConfigError::DuplicateConfigError { name: _ } => {
                HttpResponse::BadRequest().body("duplicate config")
            }
        },
    };
}
