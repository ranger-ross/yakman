use std::collections::HashMap;

use crate::{adapters::errors::CreateLabelError, StateManager, YakManError};

use actix_web::{get, put, web, HttpResponse, Responder};
use yak_man_core::model::{Label, LabelType};

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

#[get("/labels")]
pub async fn get_labels(
    state: web::Data<StateManager>,
) -> actix_web::Result<impl Responder, YakManError> {
    let service = state.get_service();

    return match service.get_labels().await {
        Ok(data) => Ok(web::Json(data)),
        Err(err) => Err(YakManError::new("Failed to load labels from storage")),
    };
}

#[put("/labels")]
pub async fn create_label(data: String, state: web::Data<StateManager>) -> HttpResponse {
    let service = state.get_service();

    let label_type: Option<LabelType> = serde_json::from_str(&data).ok();

    if let Some(label_type) = label_type {
        return match service.create_label(label_type).await {
            Ok(()) => HttpResponse::Ok().body(""),
            Err(e) => match e {
                CreateLabelError::DuplicateLabelError { name: _ } => {
                    HttpResponse::BadRequest().body("Duplicate label")
                }
                CreateLabelError::EmptyOptionsError => {
                    // TODO: This does not appear to be working
                    HttpResponse::BadRequest().body("Label must have at least 1 option")
                }
                CreateLabelError::InvalidPriorityError { prioity } => {
                    HttpResponse::BadRequest().body(format!("Invalid prioity: {prioity}"))
                }
                CreateLabelError::StorageError { message } => {
                    println!("Failed to create label, error: {message}");
                    HttpResponse::InternalServerError().body("Failed to create label")
                }
            },
        };
    }

    return HttpResponse::BadRequest().body(""); // Bad input so parse failed
}

// TODO: Standardize REST endpoint naming

#[get("/config/{config_name}/instance")]
async fn get_data_by_labels(
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let config_name = path.into_inner();
    let service = state.get_service();

    let labels: Vec<Label> = query
        .iter()
        .map(|param| Label {
            label_type: param.0.to_string(),
            value: param.1.to_string(),
        })
        .collect();

    println!("Search for config {config_name} with labels: {:?}", labels);

    return match service
        .get_config_data_by_labels(&config_name, labels)
        .await
    {
        Ok(data) => {
            if let Some(data) = data {
                HttpResponse::Ok().body(data)
            } else {
                HttpResponse::NotFound().body("Config not found")
            }
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    };
}

// TODO: Rename method to get Config by ID
#[get("/instances/{config_name}")]
async fn get_instance_by_id(
    path: web::Path<String>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let config_name = path.into_inner();
    println!("Searching for config {config_name}");
    let service = state.get_service();
    return match service.get_config_instance_metadata(&config_name).await {
        Ok(data) => match data {
            Some(data) => HttpResponse::Ok().body(serde_json::to_string(&data).unwrap()),
            None => HttpResponse::NotFound().body("Instance not found"),
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    };
}
