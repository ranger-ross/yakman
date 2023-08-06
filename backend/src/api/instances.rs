use std::collections::HashMap;

use crate::model::{Label, YakManRole};
use crate::{error::CreateConfigInstanceError, middleware::roles::YakManRoleBinding, StateManager};
use actix_web::{get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::permissions::AuthDetails;

/// Get config instances by config_name
#[utoipa::path(responses((status = 200, body = Vec<ConfigInstance>)))]
#[get("/v1/configs/{config_name}/instances")]
async fn get_instances_by_config_name(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<String>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let config_name = path.into_inner();
    let service = state.get_service();

    let config = match service.get_config(&config_name).await {
        Ok(config) => match config {
            Some(config) => config,
            None => return HttpResponse::NotFound().body("Config not found"),
        },
        Err(_) => return HttpResponse::InternalServerError().body("Failed to load config"),
    };

    let has_role = YakManRoleBinding::has_any_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
            YakManRole::Viewer,
        ],
        &config.project_uuid,
        &auth_details.permissions,
    );

    if !has_role {
        return HttpResponse::Forbidden().finish();
    }

    return match service.get_config_instance_metadata(&config_name).await {
        Ok(data) => match data {
            Some(data) => HttpResponse::Ok().body(
                serde_json::to_string(&data)
                    .expect("Failed to serialize Vec<ConfigInstance> to JSON"),
            ),
            None => HttpResponse::NotFound().body("Instance not found"),
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    };
}

/// Get config instance by instance ID
#[utoipa::path(responses((status = 200, body = ConfigInstance)))]
#[get("/v1/configs/{config_name}/instances/{instance}")]
async fn get_instance(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String)>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let (config_name, instance) = path.into_inner();
    let service = state.get_service();

    let config = match service.get_config(&config_name).await {
        Ok(config) => match config {
            Some(config) => config,
            None => return HttpResponse::NotFound().body("Config not found"),
        },
        Err(_) => return HttpResponse::InternalServerError().body("Failed to load config"),
    };

    let has_role = YakManRoleBinding::has_any_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
            YakManRole::Viewer,
        ],
        &config.project_uuid,
        &auth_details.permissions,
    );

    if !has_role {
        return HttpResponse::Forbidden().finish();
    }

    return match service.get_config_instance(&config_name, &instance).await {
        Ok(data) => match data {
            Some(data) => HttpResponse::Ok().body(
                serde_json::to_string(&data)
                    .expect("Failed to serialize Vec<ConfigInstance> to JSON"),
            ),
            None => HttpResponse::NotFound().body("Instance not found"),
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    };
}

/// Create a new config instance
#[utoipa::path(responses((status = 200, body = String)))]
#[put("/v1/configs/{config_name}/instances")]
async fn create_new_instance(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
    data: String,
    state: web::Data<StateManager>,
    req: HttpRequest,
) -> HttpResponse {
    let config_name = path.into_inner();
    let service = state.get_service();

    let labels: Vec<Label> = extract_labels(query);
    let content_type: Option<String> = get_content_type(&req);

    let config = match service.get_config(&config_name).await {
        Ok(config) => match config {
            Some(config) => config,
            None => return HttpResponse::NotFound().body("Config not found"),
        },
        Err(_) => return HttpResponse::InternalServerError().body("Failed to load config"),
    };

    let has_role = YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &config.project_uuid,
        &auth_details.permissions,
    );

    if !has_role {
        return HttpResponse::Forbidden().finish();
    }

    // TODO: do validation
    // - labels are valid
    // - not a duplicate?

    match service
        .create_config_instance(&config_name, labels, &data, content_type)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(CreateConfigInstanceError::NoConfigFound) => {
            HttpResponse::BadRequest().body("Invalid config name")
        }
        Err(CreateConfigInstanceError::StorageError { message: _ }) => {
            HttpResponse::InternalServerError().body("Failed to create config")
        }
    }
}

/// Create a update config instance
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/v1/configs/{config_name}/instances/{instance}")]
async fn update_new_instance(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String)>,
    query: web::Query<HashMap<String, String>>,
    data: String,
    state: web::Data<StateManager>,
    req: HttpRequest,
) -> HttpResponse {
    let (config_name, instance) = path.into_inner();
    let service = state.get_service();

    let labels: Vec<Label> = extract_labels(query);
    let content_type: Option<String> = get_content_type(&req);

    let config = match service.get_config(&config_name).await {
        Ok(config) => match config {
            Some(config) => config,
            None => return HttpResponse::NotFound().body("Config not found"),
        },
        Err(_) => return HttpResponse::InternalServerError().body("Failed to load config"),
    };

    let has_role = YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &config.project_uuid,
        &auth_details.permissions,
    );

    if !has_role {
        return HttpResponse::Forbidden().finish();
    }

    // TODO: do validation
    // - labels are valid
    // - not a duplicate?

    return match service
        .save_config_instance(&config_name, &instance, labels, &data, content_type)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().body("failed to create instance"),
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

fn get_content_type(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("content-type")?
        .to_str()
        .map(|s| String::from(s))
        .ok()
}
