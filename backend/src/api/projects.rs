use std::{collections::HashSet, sync::Arc};

use crate::{
    api::is_alphanumeric_kebab_case,
    error::{CreateProjectError, YakManApiError},
    middleware::roles::YakManRoleBinding,
    model::{request::CreateProjectPayload, YakManProject, YakManRole},
    services::StorageService,
};

use actix_web::{get, put, web, HttpResponse, Responder};
use actix_web_grants::permissions::AuthDetails;
use log::error;

/// Get all of the projects
#[utoipa::path(responses((status = 200, body = Vec<YakManProject>)))]
#[get("/v1/projects")]
pub async fn get_projects(
    auth_details: AuthDetails<YakManRoleBinding>,
    storage_service: web::Data<Arc<dyn StorageService>>,
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
        .any(|v| v);

    let allowed_projects: HashSet<String> = auth_details
        .permissions
        .into_iter()
        .filter_map(|p| match p {
            YakManRoleBinding::GlobalRoleBinding(_) => None,
            YakManRoleBinding::ProjectRoleBinding(r) => Some(r.project_uuid),
        })
        .collect();

    let projects: Vec<YakManProject> = storage_service
        .get_projects()
        .await?
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
    storage_service: web::Data<Arc<dyn StorageService>>,
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

    return match storage_service.create_project(&project_name).await {
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
    use super::*;
    use crate::model::YakManUserProjectRole;
    use crate::test_utils::fake_roles::FakeRoleExtractor;
    use crate::test_utils::*;
    use actix_web::{test, web::Data, App};
    use actix_web_grants::GrantsMiddleware;
    use anyhow::Result;
    use serde_json::Value;

    #[actix_web::test]
    async fn get_projects_should_return_projects() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        let project_foo_uuid = state.service.create_project("foo").await?;
        let project_bar_uuid = state.service.create_project("bar").await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_projects),
        )
        .await;
        let req = test::TestRequest::get().uri("/v1/projects").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp).await?;

        let first = &value.as_array().unwrap()[0];
        assert_eq!("foo", first["name"]);
        assert_eq!(project_foo_uuid, first["uuid"]);

        let second = &value.as_array().unwrap()[1];
        assert_eq!("bar", second["name"]);
        assert_eq!(project_bar_uuid, second["uuid"]);

        Ok(())
    }

    #[actix_web::test]
    async fn get_projects_should_not_return_projects_that_user_does_not_have() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        let _project_foo_uuid = state.service.create_project("foo").await?;
        let project_bar_uuid = state.service.create_project("bar").await?;

        let fake_extractor = FakeRoleExtractor::new(vec![YakManRoleBinding::ProjectRoleBinding(
            YakManUserProjectRole {
                project_uuid: project_bar_uuid.clone(),
                role: YakManRole::Admin,
            },
        )]);

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state))
                .wrap(GrantsMiddleware::with_extractor(fake_extractor))
                .service(get_projects),
        )
        .await;
        let req = test::TestRequest::get().uri("/v1/projects").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp).await?;

        assert_eq!(1, value.as_array().unwrap().len());

        let first = &value.as_array().unwrap()[0];
        assert_eq!("bar", first["name"]);
        assert_eq!(project_bar_uuid, first["uuid"]);

        Ok(())
    }

    #[actix_web::test]
    async fn create_project_should_create_project_if_request_is_valid() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
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
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
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

    #[actix_web::test]
    async fn create_project_should_return_bad_request_if_project_name_is_empty() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_project),
        )
        .await;
        let req = test::TestRequest::put()
            .uri("/v1/projects")
            .set_json(CreateProjectPayload {
                project_name: "".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status().as_u16();
        assert_eq!(400, status);
        Ok(())
    }
}
