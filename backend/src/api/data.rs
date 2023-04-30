use std::collections::HashMap;

use crate::StateManager;

use actix_web::{get, web, HttpRequest, HttpResponse};
use yak_man_core::model::Label;

/// Get config data by using labels
#[utoipa::path(responses((status = 200, body = String)))]
#[get("/configs/{config_name}/instances/data")]
async fn get_data_by_labels(
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let config_name = path.into_inner();
    let service = state.get_service();

    let labels: Vec<Label> = extract_labels(query);

    println!("Search for config {config_name} with labels: {:?}", labels);

    return match service
        .get_config_data_by_labels(&config_name, labels)
        .await
    {
        Ok(data) => {
            if let Some((data, content_type)) = data {
                HttpResponse::Ok().content_type(content_type).body(data)
            } else {
                HttpResponse::NotFound().body("Config not found")
            }
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    };
}

/// Get config data by instance ID
#[utoipa::path(responses((status = 200, body = String)))]
#[get("/configs/{config_name}/instances/{instance}/data")]
async fn get_instance_data(
    path: web::Path<(String, String)>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let (config_name, instance) = path.into_inner();
    let service = state.get_service();

    return match service.get_config_data(&config_name, &instance).await {
        Ok(data) => match data {
            Some((data, content_type)) => HttpResponse::Ok().content_type(content_type).body(data),
            None => HttpResponse::NotFound().body("Instance not found"),
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    };
}

fn extract_labels(query: web::Query<HashMap<String, String>>) -> Vec<Label> {
    return query
        .iter()
        .map(|param| Label {
            label_type: param.0.to_string(),
            value: param.1.to_string(),
        })
        .collect();
}