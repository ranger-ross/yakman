use crate::{api::is_alphanumeric_kebab_case, services::errors::CreateProjectError, StateManager};

use actix_web::{get, put, web, HttpResponse};
use actix_web_grants::proc_macro::has_any_role;
use log::error;
use yak_man_core::model::{request::CreateProjectPayload, YakManProject, YakManRole};

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

/// Create a new project
#[utoipa::path(request_body = CreateProjectPayload, responses((status = 200, body = String)))]
#[put("/projects")]
#[has_any_role("YakManRole::Admin", "YakManRole::Approver", type = "YakManRole")]
async fn create_project(
    payload: web::Json<CreateProjectPayload>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let payload = payload.into_inner();
    let project_name = payload.project_name.to_lowercase();

    if !is_alphanumeric_kebab_case(&project_name) {
        return HttpResponse::BadRequest()
            .body("Invalid project name. Must be alphanumeric kebab case");
    }

    let service = state.get_service();

    return match service.create_project(&project_name).await {
        Ok(()) => HttpResponse::Ok().body(""),
        Err(e) => match e {
            CreateProjectError::StorageError { message } => {
                error!("Failed to create config {project_name}, error: {message}");
                HttpResponse::InternalServerError().body("Failed to create config")
            }
            CreateProjectError::DuplicateNameError { name: _ } => {
                HttpResponse::BadRequest().body("duplicate project")
            }
        },
    };
}
