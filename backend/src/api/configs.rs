use std::collections::HashMap;

use crate::{
    services::errors::{CreateConfigError, CreateConfigInstanceError, CreateLabelError},
    StateManager, YakManError,
};

use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};
use yak_man_core::model::{Label, LabelType};

/// List of all configs
#[utoipa::path(responses((status = 200, body = Vec<Config>)))]
#[get("/configs")]
pub async fn get_configs(
    state: web::Data<StateManager>,
) -> actix_web::Result<impl Responder, YakManError> {
    let service = state.get_service();

    return match service.get_configs().await {
        Ok(data) => Ok(web::Json(data)),
        Err(err) => Err(YakManError::from(err)),
    };
}

/// Create a new config
#[utoipa::path(responses((status = 200, body = String)))]
#[put("/config/{config_name}")]
async fn create_config(path: web::Path<String>, state: web::Data<StateManager>) -> HttpResponse {
    let config_name = path.into_inner();
    let service = state.get_service();
    let result = service.create_config(&config_name).await;

    return match result {
        Ok(()) => HttpResponse::Ok().body(""),
        Err(e) => match e {
            CreateConfigError::StorageError { message } => {
                println!("Failed to create config {config_name}, error: {message}");
                HttpResponse::InternalServerError().body("Failed to create config")
            }
            CreateConfigError::DuplicateConfigError { name: _ } => {
                HttpResponse::BadRequest().body("duplicate config")
            }
        },
    };
}