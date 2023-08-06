use std::collections::HashSet;

use crate::{
    api::is_alphanumeric_kebab_case,
    error::CreateProjectError,
    middleware::roles::YakManRoleBinding,
    model::{request::CreateProjectPayload, YakManProject, YakManRole},
    StateManager,
};

use actix_web::{get, put, web, HttpResponse};
use actix_web_grants::permissions::AuthDetails;
use log::error;

/// Get all of the projects
#[utoipa::path(responses((status = 200, body = Vec<YakManProject>)))]
#[get("/v1/projects")]
pub async fn get_projects(
    auth_details: AuthDetails<YakManRoleBinding>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    if auth_details.permissions.len() == 0 {
        return HttpResponse::Forbidden().finish();
    }

    let user_has_global_role = auth_details
        .permissions
        .iter()
        .map(|p| match p {
            YakManRoleBinding::GlobalRoleBinding(_) => true,
            _ => false,
        })
        .filter(|p| p.clone())
        .collect::<Vec<bool>>()
        .len()
        > 0;

    let allowed_projects: HashSet<String> = auth_details
        .permissions
        .into_iter()
        .map(|p| match p {
            YakManRoleBinding::GlobalRoleBinding(_) => None,
            YakManRoleBinding::ProjectRoleBinding(r) => Some(r.project_uuid),
        })
        .filter(|p| p.is_some())
        .map(|p| p.unwrap())
        .collect();

    let service = state.get_service();
    let projects: Vec<YakManProject> = service
        .get_projects()
        .await
        .unwrap()
        .into_iter()
        .filter(|p| user_has_global_role || allowed_projects.contains(&p.uuid))
        .collect();

    return HttpResponse::Ok().body(serde_json::to_string(&projects).unwrap());
}

/// Create a new project
#[utoipa::path(request_body = CreateProjectPayload, responses((status = 200, body = String)))]
#[put("/v1/projects")]
async fn create_project(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: web::Json<CreateProjectPayload>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let payload = payload.into_inner();
    let project_name = payload.project_name.to_lowercase();

    let is_user_global_admin_or_approver = auth_details
        .permissions
        .into_iter()
        .filter_map(|p| match p {
            YakManRoleBinding::GlobalRoleBinding(role) => Some(role),
            YakManRoleBinding::ProjectRoleBinding(_) => None,
        })
        .filter(|role| vec![YakManRole::Admin, YakManRole::Approver].contains(role))
        .collect::<Vec<_>>()
        .len()
        > 0;

    if !is_user_global_admin_or_approver {
        return HttpResponse::Forbidden().finish();
    }

    if !is_alphanumeric_kebab_case(&project_name) {
        return HttpResponse::BadRequest()
            .body("Invalid project name. Must be alphanumeric kebab case");
    }

    let service = state.get_service();

    return match service.create_project(&project_name).await {
        Ok(()) => HttpResponse::Ok().finish(),
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
