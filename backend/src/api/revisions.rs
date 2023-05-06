use crate::StateManager;

use actix_web::{get, post, web, HttpResponse, put};
use actix_web_grants::proc_macro::has_any_role;
use yak_man_core::model::YakManRole;

/// Get all of the revisions for a config
#[utoipa::path(responses((status = 200, body = Vec<ConfigInstanceRevision>)))]
#[get("/configs/{config_name}/instances/{instance}/revisions")]
#[has_any_role(
    "YakManRole::Admin",
    "YakManRole::Approver",
    "YakManRole::Operator",
    "YakManRole::Viewer",
    type = "YakManRole"
)]
async fn get_instance_revisions(
    path: web::Path<(String, String)>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let (config_name, instance) = path.into_inner();
    let service = state.get_service();

    if let Some(data) = service
        .get_instance_revisions(&config_name, &instance)
        .await
        .unwrap()
    {
        return HttpResponse::Ok().body(serde_json::to_string(&data).unwrap());
    }
    return HttpResponse::NotFound().body("");
}

/// Submit a new revision for review
#[utoipa::path(responses((status = 200, body = String)))]
#[put("/configs/{config_name}/instances/{instance}/revisions/{revision}/submit")]
#[has_any_role(
    "YakManRole::Admin",
    "YakManRole::Approver",
    "YakManRole::Operator",
    type = "YakManRole"
)]
async fn submit_instance_revision(
    path: web::Path<(String, String, String)>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let (config_name, instance, revision) = path.into_inner();
    let service = state.get_service();

    return match service
        .update_instance_current_revision(&config_name, &instance, &revision)
        .await
    {
        Ok(_) => HttpResponse::Ok().body(""),
        Err(_) => HttpResponse::InternalServerError().body("failed to update instance"),
    };
}

/// Approves and applies a revision to a config instance
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/configs/{config_name}/instances/{instance}/revisions/{revision}/approve")]
#[has_any_role(
    "YakManRole::Admin",
    "YakManRole::Approver",
    type = "YakManRole"
)]
async fn approve_pending_instance_revision(
    path: web::Path<(String, String, String)>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let (config_name, instance, revision) = path.into_inner();
    let service = state.get_service();

    return match service
        .approve_pending_instance_revision(&config_name, &instance, &revision)
        .await
    {
        Ok(_) => HttpResponse::Ok().body(""),
        Err(_) => HttpResponse::InternalServerError().body("failed to update instance"),
    };
}
