use std::sync::Arc;

use crate::error::YakManApiError;
use crate::middleware::roles::YakManRoleBinding;
use crate::model::YakManRole;
use crate::services::StorageService;
use actix_web::{get, web, HttpResponse, Responder};
use actix_web_grants::authorities::AuthDetails;

/// Get config data by instance ID
#[utoipa::path(responses((status = 200, body = String)))]
#[get("/v1/configs/{config_id}/instances/{instance}/data")]
async fn get_instance_data(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String)>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let (config_id, instance) = path.into_inner();

    let config = match storage_service.get_config(&config_id).await {
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
        &config.project_id,
        &auth_details.authorities,
    );

    if !has_role {
        return Err(YakManApiError::forbidden());
    }

    let data = storage_service
        .get_config_data(&config_id, &instance)
        .await?;

    return match data {
        Some((data, content_type)) => Ok(HttpResponse::Ok().content_type(content_type).body(data)),
        None => Err(YakManApiError::not_found("Instance not found")),
    };
}

/// Get config data by instance ID and revision ID
#[utoipa::path(responses((status = 200, body = String)))]
#[get("/v1/configs/{config_id}/instances/{instance}/revisions/{revision}/data")]
async fn get_revision_data(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String, String)>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let (config_id, _, revision) = path.into_inner();

    let config = match storage_service.get_config(&config_id).await {
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
        &config.project_id,
        &auth_details.authorities,
    );

    if !has_role {
        return Err(YakManApiError::forbidden());
    }

    let data = storage_service
        .get_data_by_revision(&config_id, &revision)
        .await?;

    return match data {
        Some((data, content_type)) => Ok(HttpResponse::Ok().content_type(content_type).body(data)),
        None => Err(YakManApiError::not_found("Instance not found")),
    };
}
