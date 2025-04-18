use std::sync::Arc;

use crate::error::{RollbackRevisionError, YakManApiError};
use crate::middleware::roles::YakManRoleBinding;
use crate::middleware::YakManPrinciple;
use crate::model::response::RevisionPayload;
use crate::model::{ConfigInstanceRevision, YakManRole};
use crate::services::StorageService;
use actix_web::{get, post, web, HttpResponse, Responder};
use actix_web_grants::authorities::AuthDetails;
use serde::Deserialize;
use utoipa::ToSchema;

/// Get all of the revisions for a config
#[utoipa::path(responses((status = 200, body = Vec<ConfigInstanceRevision>)))]
#[get("/v1/configs/{config_id}/instances/{instance}/revisions")]
async fn get_instance_revisions(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String)>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let (config_id, instance) = path.into_inner();

    let config = storage_service
        .get_config(&config_id)
        .await
        .unwrap()
        .unwrap(); // TODO: handle these unwraps

    if !YakManRoleBinding::has_any_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
            YakManRole::Viewer,
        ],
        &config.project_id,
        &auth_details.authorities,
    ) {
        return Err(YakManApiError::forbidden());
    }

    if let Some(data) = storage_service
        .get_instance_revisions(&config_id, &instance)
        .await?
    {
        return Ok(web::Json(data));
    }
    return Err(YakManApiError::not_found("revision not found"));
}

#[derive(Debug, Deserialize, PartialEq, Eq, ToSchema)]
pub enum ReviewResult {
    Approve,
    ApproveAndApply,
    Reject,
}

/// Updates a revsion based on a review result.
#[utoipa::path(responses((status = 200, body = ())))]
#[post("/v1/configs/{config_id}/instances/{instance}/revisions/{revision}/review/{result}")]
async fn review_pending_instance_revision(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String, String, ReviewResult)>,
    storage_service: web::Data<Arc<dyn StorageService>>,
    principle: YakManPrinciple,
) -> Result<impl Responder, YakManApiError> {
    let (config_id, instance, revision, result) = path.into_inner();

    let config = storage_service
        .get_config(&config_id)
        .await
        .unwrap()
        .unwrap();

    if !YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &config.project_id,
        &auth_details.authorities,
    ) {
        return Err(YakManApiError::forbidden());
    }

    let Some(reviewer_user_id) = principle.user_id else {
        return Err(YakManApiError::forbidden());
    };

    match result {
        ReviewResult::ApproveAndApply | ReviewResult::Approve => {
            return match storage_service
                .approve_instance_revision(&config_id, &instance, &revision, &reviewer_user_id)
                .await
            {
                Ok(_) => {
                    if result == ReviewResult::ApproveAndApply {
                        return match storage_service
                            .apply_instance_revision(
                                &config_id,
                                &instance,
                                &revision,
                                &reviewer_user_id,
                            )
                            .await
                        {
                            Ok(_) => Ok(HttpResponse::Ok().finish()),
                            Err(_) => {
                                Err(YakManApiError::server_error("failed to update instance"))
                            }
                        };
                    }

                    return Ok(HttpResponse::Ok().finish());
                }
                Err(_) => Err(YakManApiError::server_error("failed to update instance")),
            };
        }
        ReviewResult::Reject => {
            return match storage_service
                .reject_instance_revision(&config_id, &instance, &revision, &reviewer_user_id)
                .await
            {
                Ok(_) => Ok(HttpResponse::Ok().finish()),
                Err(_) => Err(YakManApiError::server_error("failed to update instance")),
            };
        }
    }
}

/// Applies an approved revision
#[utoipa::path(responses((status = 200, body = ())))]
#[post("/v1/configs/{config_id}/instances/{instance}/revisions/{revision}/apply")]
async fn apply_instance_revision(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String, String)>,
    storage_service: web::Data<Arc<dyn StorageService>>,
    principle: YakManPrinciple,
) -> Result<impl Responder, YakManApiError> {
    let (config_id, instance, revision) = path.into_inner();

    let config = storage_service
        .get_config(&config_id)
        .await
        .unwrap()
        .unwrap(); // todo: handle these unwraps

    if !YakManRoleBinding::has_any_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
        ],
        &config.project_id,
        &auth_details.authorities,
    ) {
        return Err(YakManApiError::forbidden());
    }

    let Some(reviewer_user_id) = principle.user_id else {
        return Err(YakManApiError::forbidden());
    };

    return match storage_service
        .apply_instance_revision(&config_id, &instance, &revision, &reviewer_user_id)
        .await
    {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Err(YakManApiError::server_error("failed to update instance")),
    };
}

/// Rollback an instance a previous revision (by cloning the revision)
#[utoipa::path(responses((status = 200, body = RevisionPayload)))]
#[post("/v1/configs/{config_id}/instances/{instance}/revisions/{revision}/rollback")]
async fn rollback_instance_revision(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String, String)>,
    storage_service: web::Data<Arc<dyn StorageService>>,
    principle: YakManPrinciple,
) -> Result<impl Responder, YakManApiError> {
    let (config_id, instance, revision) = path.into_inner();

    let config = storage_service
        .get_config(&config_id)
        .await?
        .ok_or(RollbackRevisionError::InvalidConfig)?;

    if !YakManRoleBinding::has_any_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
        ],
        &config.project_id,
        &auth_details.authorities,
    ) {
        return Err(YakManApiError::forbidden());
    }

    let rollback_by_user_id = principle.user_id.ok_or(YakManApiError::forbidden())?;

    let new_revision = storage_service
        .rollback_instance_revision(&config_id, &instance, &revision, &rollback_by_user_id)
        .await?;

    Ok(web::Json(RevisionPayload {
        revision: new_revision,
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
