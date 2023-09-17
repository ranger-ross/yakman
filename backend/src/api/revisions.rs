use crate::middleware::YakManPrinciple;
use crate::model::YakManRole;
use crate::{middleware::roles::YakManRoleBinding, StateManager};
use actix_web::{get, post, put, web, HttpResponse};
use actix_web_grants::permissions::AuthDetails;
use serde::Deserialize;

/// Get all of the revisions for a config
#[utoipa::path(responses((status = 200, body = Vec<ConfigInstanceRevision>)))]
#[get("/v1/configs/{config_name}/instances/{instance}/revisions")]
async fn get_instance_revisions(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String)>,
    state: web::Data<StateManager>,
) -> HttpResponse {
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
        return HttpResponse::Forbidden().finish();
    }

    if let Some(data) = service
        .get_instance_revisions(&config_name, &instance)
        .await
        .unwrap()
    {
        return HttpResponse::Ok().body(serde_json::to_string(&data).unwrap());
    }
    return HttpResponse::NotFound().finish();
}

/// Submit a new revision for review
#[utoipa::path(responses((status = 200, body = String)))]
#[put("/v1/configs/{config_name}/instances/{instance}/revisions/{revision}/submit")]
async fn submit_instance_revision(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String, String)>,
    state: web::Data<StateManager>,
) -> HttpResponse {
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
        return HttpResponse::Forbidden().finish();
    }

    return match service
        .update_instance_current_revision(&config_name, &instance, &revision)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().body("failed to update instance"),
    };
}

#[derive(Debug, Deserialize)]

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
) -> HttpResponse {
    let (config_name, instance, revision, result) = path.into_inner();
    let service = state.get_service();

    let config = service.get_config(&config_name).await.unwrap().unwrap();

    if !YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &config.project_uuid,
        &auth_details.permissions,
    ) {
        return HttpResponse::Forbidden().finish();
    }

    let reviewer_uuid = principle.user_uuid;
    if reviewer_uuid.is_none() {
        return HttpResponse::Forbidden().finish();
    }

    match result {
        ReviewResult::Approve => {
            return match service
                .approve_pending_instance_revision(
                    &config_name,
                    &instance,
                    &revision,
                    &reviewer_uuid.unwrap(),
                )
                .await
            {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(_) => HttpResponse::InternalServerError().body("failed to update instance"),
            };
        }
        ReviewResult::ApproveAndApply => todo!(),
        ReviewResult::Reject => todo!(),
    }
}
