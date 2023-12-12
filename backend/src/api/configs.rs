use crate::model::{
    request::{CreateConfigPayload, DeleteConfigPayload},
    YakManRole,
};
use crate::{
    api::is_alphanumeric_kebab_case,
    error::YakManApiError,
    error::{CreateConfigError, DeleteConfigError},
    middleware::roles::YakManRoleBinding,
    StateManager,
};
use actix_web::{delete, get, put, web, Responder};
use actix_web_grants::permissions::AuthDetails;
use log::error;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize)]
pub struct GetConfigsQuery {
    pub project: Option<String>,
}

/// List of all configs
#[utoipa::path(responses((status = 200, body = Vec<Config>)))]
#[get("/v1/configs")]
pub async fn get_configs(
    auth_details: AuthDetails<YakManRoleBinding>,
    query: web::Query<GetConfigsQuery>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let project_uuid = query.project.to_owned();
    let has_global_role = YakManRoleBinding::has_any_global_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
            YakManRole::Viewer,
        ],
        &auth_details.permissions,
    );

    if let Some(project_uuid) = &project_uuid {
        if !has_global_role
            && !YakManRoleBinding::has_any_role(
                vec![
                    YakManRole::Admin,
                    YakManRole::Approver,
                    YakManRole::Operator,
                    YakManRole::Viewer,
                ],
                project_uuid,
                &auth_details.permissions,
            )
        {
            return Err(YakManApiError::forbidden());
        }
    }

    let allowed_projects: HashSet<String> = auth_details
        .permissions
        .into_iter()
        .filter_map(|p| match p {
            YakManRoleBinding::ProjectRoleBinding(role) => Some(role.project_uuid),
            _ => None,
        })
        .collect();

    let service = state.get_service();
    let data = service.get_visible_configs(project_uuid).await?;

    if has_global_role {
        return Ok(web::Json(data));
    }

    let filtered_data = data
        .into_iter()
        .filter(|c| allowed_projects.contains(&c.project_uuid))
        .collect();

    return Ok(web::Json(filtered_data));
}

/// Create a new config
#[utoipa::path(request_body = CreateConfigPayload, responses((status = 200, body = String)))]
#[put("/v1/configs")]
async fn create_config(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: web::Json<CreateConfigPayload>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let payload = payload.into_inner();
    let config_name = payload.config_name.to_lowercase();
    let project_uuid = payload.project_uuid;

    if !YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &project_uuid,
        &auth_details.permissions,
    ) {
        return Err(YakManApiError::forbidden());
    }

    if config_name.is_empty() {
        return Err(YakManApiError::bad_request(
            "Invalid config name. Must not be empty",
        ));
    }

    if !is_alphanumeric_kebab_case(&config_name) {
        return Err(YakManApiError::bad_request(
            "Invalid config name. Must be alphanumeric kebab case",
        ));
    }

    let service = state.get_service();

    let projects = match service.get_projects().await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to load projects, error: {e:?}");
            return Err(YakManApiError::server_error("Failed to create config"));
        }
    };

    if projects
        .into_iter()
        .find(|p| p.uuid == project_uuid)
        .is_none()
    {
        return Err(YakManApiError::bad_request("Project does not exist"));
    }

    let result: Result<(), CreateConfigError> =
        service.create_config(&config_name, &project_uuid).await;

    return match result {
        Ok(()) => Ok(web::Json(config_name)),
        Err(e) => match e {
            CreateConfigError::StorageError { message } => {
                error!("Failed to create config {config_name}, error: {message}");
                Err(YakManApiError::server_error("Failed to create config"))
            }
            CreateConfigError::DuplicateConfigError { name: _ } => {
                Err(YakManApiError::bad_request("duplicate config"))
            }
        },
    };
}

/// Hide a config instance from the UI and API (data not deleted)
#[utoipa::path(request_body = DeleteConfigPayload, responses((status = 200, body = String)))]
#[delete("/v1/configs")]
async fn delete_config(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: web::Json<DeleteConfigPayload>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let payload = payload.into_inner();
    let config_name = payload.config_name.to_lowercase();
    let project_uuid = payload.project_uuid;

    if !YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin],
        &project_uuid,
        &auth_details.permissions,
    ) {
        return Err(YakManApiError::forbidden());
    }

    if !is_alphanumeric_kebab_case(&config_name) {
        return Err(YakManApiError::bad_request(
            "Invalid config name. Must be alphanumeric kebab case",
        ));
    }

    let service = state.get_service();

    let result: Result<(), DeleteConfigError> = service.delete_config(&config_name).await;

    return match result {
        Ok(()) => Ok(web::Json(())),
        Err(e) => match e {
            DeleteConfigError::StorageError { message } => {
                error!("Failed to create config {config_name}, error: {message}");
                Err(YakManApiError::server_error("Failed to delete config"))
            }
            DeleteConfigError::ConfigDoesNotExistError => {
                Err(YakManApiError::bad_request("config does not exist"))
            }
        },
    };
}
