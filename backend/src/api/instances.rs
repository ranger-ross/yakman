use std::collections::HashMap;

use crate::error::{SaveConfigInstanceError, YakManApiError};
use crate::middleware::YakManPrinciple;
use crate::model::response::{InstancePayload, RevisionPayload};
use crate::model::{YakManLabel, YakManRole};
use crate::{error::CreateConfigInstanceError, middleware::roles::YakManRoleBinding, StateManager};
use actix_web::{delete, get, post, put, web, HttpRequest, Responder};
use actix_web_grants::permissions::AuthDetails;

/// Get config instances by config_name
#[utoipa::path(responses((status = 200, body = Vec<ConfigInstance>)))]
#[get("/v1/configs/{config_name}/instances")]
async fn get_instances_by_config_name(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<String>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let config_name = path.into_inner();
    let service = state.get_service();

    let config = match service.get_config(&config_name).await {
        Ok(config) => match config {
            Some(config) => config,
            None => return Err(YakManApiError::not_found("Config not found")),
        },
        Err(_) => return Err(YakManApiError::server_error("Failed to load config")),
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
        return Err(YakManApiError::forbidden());
    }

    let data = service.get_config_instance_metadata(&config_name).await?;

    return match data {
        Some(data) => Ok(web::Json(data)),
        None => Err(YakManApiError::not_found("Instance not found")),
    };
}

/// Get config instance by instance ID
#[utoipa::path(responses((status = 200, body = ConfigInstance)))]
#[get("/v1/configs/{config_name}/instances/{instance}")]
async fn get_instance(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String)>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let (config_name, instance) = path.into_inner();
    let service = state.get_service();

    let config = match service.get_config(&config_name).await? {
        Some(config) => config,
        None => return Err(YakManApiError::not_found("Config not found")),
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
        return Err(YakManApiError::forbidden());
    }

    return match service.get_config_instance(&config_name, &instance).await? {
        Some(data) => Ok(web::Json(data)),
        None => Err(YakManApiError::not_found("Instance not found")),
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
    principle: YakManPrinciple,
) -> Result<impl Responder, YakManApiError> {
    let config_name = path.into_inner();
    let service = state.get_service();

    let labels: Vec<YakManLabel> = extract_labels(query);
    let content_type: Option<String> = get_content_type(&req);

    let config = match service.get_config(&config_name).await {
        Ok(config) => match config {
            Some(config) => config,
            None => return Err(YakManApiError::not_found("Config not found")),
        },
        Err(_) => return Err(YakManApiError::server_error("Failed to load config")),
    };

    let has_role = YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &config.project_uuid,
        &auth_details.permissions,
    );

    if !has_role {
        return Err(YakManApiError::forbidden());
    }

    let creator_uuid = principle.user_uuid.ok_or(YakManApiError::forbidden())?;

    match service
        .create_config_instance(&config_name, labels, &data, content_type, &creator_uuid)
        .await
    {
        Ok(instance) => Ok(web::Json(InstancePayload { instance: instance })),
        Err(CreateConfigInstanceError::NoConfigFound) => {
            Err(YakManApiError::bad_request("Invalid config name"))
        }
        Err(CreateConfigInstanceError::InvalidLabel) => {
            Err(YakManApiError::bad_request("Invalid label"))
        }
        Err(CreateConfigInstanceError::StorageError { message: _ }) => {
            Err(YakManApiError::server_error("Failed to create config"))
        }
    }
}

/// Submit changes for an approval (creates a new revision)
#[utoipa::path(responses((status = 200, body = RevisionPayload)))]
#[post("/v1/configs/{config_name}/instances/{instance}")]
async fn update_new_instance(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String)>,
    query: web::Query<HashMap<String, String>>,
    data: String,
    state: web::Data<StateManager>,
    req: HttpRequest,
    principle: YakManPrinciple,
) -> Result<impl Responder, YakManApiError> {
    let (config_name, instance) = path.into_inner();
    let service = state.get_service();

    let labels: Vec<YakManLabel> = extract_labels(query);
    let content_type: Option<String> = get_content_type(&req);

    let config = match service.get_config(&config_name).await {
        Ok(config) => match config {
            Some(config) => config,
            None => return Err(YakManApiError::not_found("Config not found")),
        },
        Err(_) => return Err(YakManApiError::server_error("Failed to load config")),
    };

    let has_role = YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &config.project_uuid,
        &auth_details.permissions,
    );

    if !has_role {
        return Err(YakManApiError::forbidden());
    }

    let creator_uuid = principle.user_uuid.ok_or(YakManApiError::forbidden())?;

    let new_revsion = service
        .submit_new_instance_revision(
            &config_name,
            &instance,
            labels,
            &data,
            content_type,
            &creator_uuid,
        )
        .await
        .map_err(|e| match e {
            SaveConfigInstanceError::InvalidConfig => YakManApiError::bad_request("invalid config"),
            SaveConfigInstanceError::InvalidInstance => {
                YakManApiError::bad_request("invalid instance")
            }
            SaveConfigInstanceError::InvalidLabel => YakManApiError::bad_request("invalid label"),
            SaveConfigInstanceError::StorageError { message: _ } => {
                YakManApiError::server_error("failed to create instance")
            }
        })?;
    Ok(web::Json(RevisionPayload {
        revision: new_revsion,
    }))
}

/// Delete a config instance
#[utoipa::path(responses((status = 200, body = String)))]
#[delete("/v1/configs/{config_name}/instances/{instance}")]
async fn delete_instance(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String)>,
    state: web::Data<StateManager>,
    req: HttpRequest,
    principle: YakManPrinciple,
) -> Result<impl Responder, YakManApiError> {
    let (config_name, instance) = path.into_inner();
    let service = state.get_service();

    let config = match service.get_config(&config_name).await {
        Ok(config) => match config {
            Some(config) => config,
            None => return Err(YakManApiError::not_found("Config not found")),
        },
        Err(_) => return Err(YakManApiError::server_error("Failed to load config")),
    };

    let has_role = YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin],
        &config.project_uuid,
        &auth_details.permissions,
    );

    if !has_role {
        return Err(YakManApiError::forbidden());
    }

    service
        .delete_instance(
            &config_name,
            &instance
        )
        .await?;
        // .map_err(|e| match e {
        //     SaveConfigInstanceError::InvalidConfig => YakManApiError::bad_request("invalid config"),
        //     SaveConfigInstanceError::InvalidInstance => {
        //         YakManApiError::bad_request("invalid instance")
        //     }
        //     SaveConfigInstanceError::InvalidLabel => YakManApiError::bad_request("invalid label"),
        //     SaveConfigInstanceError::StorageError { message: _ } => {
        //         YakManApiError::server_error("failed to create instance")
        //     }
        // })?;
    Ok(web::Json(()))
}

fn extract_labels(query: web::Query<HashMap<String, String>>) -> Vec<YakManLabel> {
    return query
        .iter()
        .map(|param| YakManLabel {
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
