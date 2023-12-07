use crate::error::YakManApiError;
use crate::model::YakManRole;
use crate::{middleware::roles::YakManRoleBinding, StateManager};
use actix_web::{get, web, HttpResponse, Responder};
use actix_web_grants::permissions::AuthDetails;

/// Get config data by instance ID
#[utoipa::path(responses((status = 200, body = String)))]
#[get("/v1/configs/{config_name}/instances/{instance}/data")]
async fn get_instance_data(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String)>,
    state: web::Data<StateManager>,
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

    let data = service.get_config_data(&config_name, &instance).await?;

    return match data {
        Some((data, content_type)) => Ok(HttpResponse::Ok().content_type(content_type).body(data)),
        None => Err(YakManApiError::not_found("Instance not found")),
    };
}

/// Get config data by instance ID and revision ID
#[utoipa::path(responses((status = 200, body = String)))]
#[get("/v1/configs/{config_name}/instances/{instance}/revisions/{revision}/data")]
async fn get_revision_data(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String, String)>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let (config_name, _, revision) = path.into_inner();
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

    let data = service.get_data_by_revision(&config_name, &revision).await?;

    return match data {
        Some((data, content_type)) => Ok(HttpResponse::Ok().content_type(content_type).body(data)),
        None => Err(YakManApiError::not_found("Instance not found")),
    };
}
