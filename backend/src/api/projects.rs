use crate::StateManager;

use actix_web::{get, web, HttpResponse};
use actix_web_grants::proc_macro::has_any_role;
use yak_man_core::model::{YakManProject, YakManRole};

/// Get all of the projects
#[utoipa::path(responses((status = 200, body = Vec<YakManProject>)))]
#[get("/v1/projects")]
#[has_any_role(
    "YakManRole::Admin",
    "YakManRole::Approver",
    "YakManRole::Operator",
    "YakManRole::Viewer",
    type = "YakManRole"
)]
pub async fn get_projects(state: web::Data<StateManager>) -> HttpResponse {
    let service = state.get_service();
    let projects: Vec<YakManProject> = service.get_projects().await.unwrap();
    return HttpResponse::Ok().body(serde_json::to_string(&projects).unwrap());
}
