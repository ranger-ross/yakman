use std::collections::HashSet;

use crate::{
    api::is_alphanumeric_kebab_case,
    error::{CreateProjectError, YakManApiError},
    middleware::roles::YakManRoleBinding,
    model::{request::CreateProjectPayload, YakManProject, YakManRole},
    StateManager,
};

use actix_web::{get, put, web, HttpResponse, Responder};
use actix_web_grants::permissions::AuthDetails;
use log::error;

/// Get all of the projects
#[utoipa::path(responses((status = 200, body = Vec<YakManProject>)))]
#[get("/v1/projects")]
pub async fn get_projects(
    auth_details: AuthDetails<YakManRoleBinding>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    if auth_details.permissions.len() == 0 {
        return Err(YakManApiError::forbidden());
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

    return Ok(web::Json(projects));
}

/// Create a new project
#[utoipa::path(request_body = CreateProjectPayload, responses((status = 200, body = String)))]
#[put("/v1/projects")]
async fn create_project(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: web::Json<CreateProjectPayload>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
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
        return Err(YakManApiError::forbidden());
    }

    if project_name.is_empty() {
        return Err(YakManApiError::bad_request(
            "Invalid project name. Must not be empty",
        ));
    }

    if !is_alphanumeric_kebab_case(&project_name) {
        return Err(YakManApiError::bad_request(
            "Invalid project name. Must be alphanumeric kebab case",
        ));
    }

    let service = state.get_service();

    return match service.create_project(&project_name).await {
        Ok(project_uuid) => Ok(HttpResponse::Ok().body(project_uuid)),
        Err(e) => match e {
            CreateProjectError::StorageError { message } => {
                error!("Failed to create config {project_name}, error: {message}");
                Err(YakManApiError::server_error("Failed to create config"))
            }
            CreateProjectError::DuplicateNameError { name: _ } => {
                Err(YakManApiError::bad_request("duplicate project"))
            }
        },
    };
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{dev::ServiceRequest, test, web::Data, App, Error};
    use actix_web_grants::GrantsMiddleware;

    use crate::{
        adapters::{in_memory::InMemoryStorageAdapter, KVStorageAdapter},
        auth::{oauth_service::MockOauthService, token::MockTokenService},
        services::kv_storage_service::KVStorageService,
    };
    use anyhow::Result;

    use super::*;

    #[actix_web::test]
    async fn create_project_should_create_project_if_request_is_valid() -> Result<()> {
        prepare_for_test()?;

        let state = test_state_manager().await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state))
                .wrap(GrantsMiddleware::with_extractor(admin_role))
                .service(create_project),
        )
        .await;
        let req = test::TestRequest::put()
            .uri("/v1/projects")
            .set_json(CreateProjectPayload {
                project_name: "valid-project-name".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        Ok(())
    }

    #[actix_web::test]
    async fn create_project_should_return_bad_request_if_project_name_is_invalid() -> Result<()> {
        prepare_for_test()?;

        let state = test_state_manager().await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state))
                .wrap(GrantsMiddleware::with_extractor(admin_role))
                .service(create_project),
        )
        .await;
        let req = test::TestRequest::put()
            .uri("/v1/projects")
            .set_json(CreateProjectPayload {
                project_name: "this is not a valid name".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status().as_u16();
        assert_eq!(400, status);
        Ok(())
    }

    pub async fn admin_role(_req: &ServiceRequest) -> Result<Vec<YakManRoleBinding>, Error> {
        return Ok(vec![YakManRoleBinding::GlobalRoleBinding(
            YakManRole::Admin,
        )]);
    }

    fn prepare_for_test() -> Result<()> {
        dotenv::dotenv()?;
        env_logger::init();

        Ok(())
    }

    async fn test_state_manager() -> Result<StateManager> {
        let adapter = InMemoryStorageAdapter::new();
        adapter.initialize_yakman_storage().await?;
        let service: KVStorageService = KVStorageService {
            adapter: Box::new(adapter),
        };

        let token_service = MockTokenService::new();
        let oauth_service = MockOauthService::new();

        return Ok(StateManager {
            jwt_service: Arc::new(token_service),
            oauth_service: Arc::new(oauth_service),
            service: Arc::new(service),
        });
    }
}
