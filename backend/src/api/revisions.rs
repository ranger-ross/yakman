use crate::error::{RollbackRevisionError, YakManApiError};
use crate::middleware::YakManPrinciple;
use crate::model::response::RevisionPayload;
use crate::model::YakManRole;
use crate::{middleware::roles::YakManRoleBinding, StateManager};
use actix_web::{get, post, web, Responder};
use actix_web_grants::permissions::AuthDetails;
use serde::Deserialize;

/// Get all of the revisions for a config
#[utoipa::path(responses((status = 200, body = Vec<ConfigInstanceRevision>)))]
#[get("/v1/configs/{config_name}/instances/{instance}/revisions")]
async fn get_instance_revisions(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String)>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let (config_name, instance) = path.into_inner();
    let service = state.get_service();

    let config = service.get_config(&config_name).await.unwrap().unwrap();

    if !YakManRoleBinding::has_any_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
            YakManRole::Viewer,
        ],
        &config.project_uuid,
        &auth_details.permissions,
    ) {
        return Err(YakManApiError::forbidden());
    }

    if let Some(data) = service
        .get_instance_revisions(&config_name, &instance)
        .await?
    {
        return Ok(web::Json(data));
    }
    return Err(YakManApiError::not_found("revision not found"));
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum ReviewResult {
    Approve,
    ApproveAndApply,
    Reject,
}

/// Updates a revsion based on a review result.
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/v1/configs/{config_name}/instances/{instance}/revisions/{revision}/review/{result}")]
async fn review_pending_instance_revision(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String, String, ReviewResult)>,
    state: web::Data<StateManager>,
    principle: YakManPrinciple,
) -> Result<impl Responder, YakManApiError> {
    let (config_name, instance, revision, result) = path.into_inner();
    let service = state.get_service();

    let config = service.get_config(&config_name).await.unwrap().unwrap();

    if !YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &config.project_uuid,
        &auth_details.permissions,
    ) {
        return Err(YakManApiError::forbidden());
    }

    let reviewer_uuid = principle.user_uuid;
    if reviewer_uuid.is_none() {
        return Err(YakManApiError::forbidden());
    }
    let reviewer_uuid = reviewer_uuid.unwrap();

    match result {
        ReviewResult::ApproveAndApply | ReviewResult::Approve => {
            return match service
                .approve_instance_revision(&config_name, &instance, &revision, &reviewer_uuid)
                .await
            {
                Ok(_) => {
                    if result == ReviewResult::ApproveAndApply {
                        return match service
                            .apply_instance_revision(
                                &config_name,
                                &instance,
                                &revision,
                                &reviewer_uuid,
                            )
                            .await
                        {
                            Ok(_) => Ok(web::Json(())),
                            Err(_) => {
                                Err(YakManApiError::server_error("failed to update instance"))
                            }
                        };
                    }

                    return Ok(web::Json(()));
                }
                Err(_) => Err(YakManApiError::server_error("failed to update instance")),
            };
        }
        ReviewResult::Reject => {
            return match service
                .reject_instance_revision(&config_name, &instance, &revision, &reviewer_uuid)
                .await
            {
                Ok(_) => Ok(web::Json(())),
                Err(_) => Err(YakManApiError::server_error("failed to update instance")),
            };
        }
    }
}

/// Applies an approved revision
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/v1/configs/{config_name}/instances/{instance}/revisions/{revision}/apply")]
async fn apply_instance_revision(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String, String)>,
    state: web::Data<StateManager>,
    principle: YakManPrinciple,
) -> Result<impl Responder, YakManApiError> {
    let (config_name, instance, revision) = path.into_inner();
    let service = state.get_service();

    let config = service.get_config(&config_name).await.unwrap().unwrap();

    if !YakManRoleBinding::has_any_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
        ],
        &config.project_uuid,
        &auth_details.permissions,
    ) {
        return Err(YakManApiError::forbidden());
    }

    let reviewer_uuid = principle.user_uuid;
    if reviewer_uuid.is_none() {
        return Err(YakManApiError::forbidden());
    }
    let reviewer_uuid = reviewer_uuid.unwrap();

    return match service
        .apply_instance_revision(&config_name, &instance, &revision, &reviewer_uuid)
        .await
    {
        Ok(_) => Ok(web::Json(())),
        Err(_) => Err(YakManApiError::server_error("failed to update instance")),
    };
}

/// Rollback an instance a previous revision (by cloning the revision)
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/v1/configs/{config_name}/instances/{instance}/revisions/{revision}/rollback")]
async fn rollback_instance_revision(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String, String)>,
    state: web::Data<StateManager>,
    principle: YakManPrinciple,
) -> Result<impl Responder, YakManApiError> {
    let (config_name, instance, revision) = path.into_inner();
    let service = state.get_service();

    let config = service
        .get_config(&config_name)
        .await?
        .ok_or(RollbackRevisionError::InvalidConfig)?;

    if !YakManRoleBinding::has_any_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
        ],
        &config.project_uuid,
        &auth_details.permissions,
    ) {
        return Err(YakManApiError::forbidden());
    }

    let rollback_by_uuid = principle.user_uuid.ok_or(YakManApiError::forbidden())?;

    let new_revision = service
        .rollback_instance_revision(&config_name, &instance, &revision, &rollback_by_uuid)
        .await?;

    Ok(web::Json(RevisionPayload {
        revision: new_revision
    }))
}

impl From<RollbackRevisionError> for YakManApiError {
    fn from(value: RollbackRevisionError) -> Self {
        return match value {
            RollbackRevisionError::InvalidConfig => YakManApiError::bad_request("Invalid Config"),
            RollbackRevisionError::InvalidInstance => {
                YakManApiError::bad_request("Invalid Config Instance")
            }
            RollbackRevisionError::InvalidRevision => {
                YakManApiError::bad_request("Invalid Revision")
            }
            RollbackRevisionError::StorageError { message } => {
                log::error!("Error while rolling back revision {message}");
                YakManApiError::server_error("Storage error")
            }
        };
    }
}
