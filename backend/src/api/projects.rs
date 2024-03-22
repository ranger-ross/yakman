use std::{collections::HashSet, sync::Arc};

use crate::{
    api::is_alphanumeric_kebab_case,
    error::{CreateProjectError, YakManApiError},
    middleware::roles::YakManRoleBinding,
    model::{
        request::{CreateProjectPayload, ProjectNotificationType},
        YakManProject, YakManRole,
    },
    services::StorageService,
    settings,
};

use actix_web::{get, put, web, HttpResponse, Responder};
use actix_web_grants::authorities::AuthDetails;
use log::error;
use url::Url;

/// Get all of the projects (user has access to)
#[utoipa::path(responses((status = 200, body = Vec<YakManProject>)))]
#[get("/v1/projects")]
pub async fn get_projects(
    auth_details: AuthDetails<YakManRoleBinding>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    if auth_details.authorities.len() == 0 {
        return Err(YakManApiError::forbidden());
    }

    let user_has_global_role = auth_details
        .authorities
        .iter()
        .map(|p| match p {
            YakManRoleBinding::GlobalRoleBinding(_) => true,
            _ => false,
        })
        .any(|v| v);

    let allowed_projects: HashSet<String> = auth_details
        .authorities
        .iter()
        .filter_map(|p| match p {
            YakManRoleBinding::GlobalRoleBinding(_) => None,
            YakManRoleBinding::ProjectRoleBinding(r) => Some(r.project_uuid.clone()),
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

/// Get project by uuid
#[utoipa::path(responses((status = 200, body = YakManProjectDetails)))]
#[get("/v1/projects/{uuid}")]
pub async fn get_project(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<String>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    if auth_details.authorities.len() == 0 {
        return Err(YakManApiError::forbidden());
    }

    let project_uuid: String = path.into_inner();
    let has_role = YakManRoleBinding::has_any_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
            YakManRole::Viewer,
        ],
        &project_uuid,
        &auth_details.authorities,
    );

    if !has_role {
        return Err(YakManApiError::forbidden());
    }

    let Some(details) = storage_service.get_project_details(&project_uuid).await? else {
        return Err(YakManApiError::not_found("Project not found"));
    };

    return Ok(web::Json(details));
}

/// Create a new project
#[utoipa::path(request_body = CreateProjectPayload, responses((status = 200, body = (), content_type = [])))]
#[put("/v1/projects")]
async fn create_project(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: web::Json<CreateProjectPayload>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let payload = payload.into_inner();
    let project_name = payload.project_name.to_lowercase();

    let is_user_global_admin_or_approver = auth_details
        .authorities
        .iter()
        .filter_map(|p| match p {
            YakManRoleBinding::GlobalRoleBinding(role) => Some(role.clone()),
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

    // Validate notification webhooks to protect against SSRF
    if let Some(notification) = &payload.notification_settings {
        match &notification.notification_type {
            ProjectNotificationType::Slack { webhook_url } => {
                let Ok(url) = Url::parse(&webhook_url) else {
                    return Err(YakManApiError::bad_request("Invalid webhook url"));
                };

                let Some(webhook_host) = url.host() else {
                    return Err(YakManApiError::bad_request("Invalid webhook url"));
                };
                let webhook_host = webhook_host.to_string();

                let is_whitelisted_host = settings::notification_whitelisted_hosts()
                    .into_iter()
                    .any(|host| host == webhook_host);

                if !is_whitelisted_host {
                    return Err(YakManApiError::bad_request("Webhook host is not permitted"));
                }
            }
        }
    }

    return match storage_service
        .create_project(&project_name, payload.notification_settings)
        .await
    {
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
    use actix_web::{test, App};
    use actix_web_grants::GrantsMiddleware;
    use anyhow::Result;
    use serde_json::Value;

    #[actix_web::test]
    async fn get_projects_should_return_projects() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let project_foo_uuid = storage_service.create_project("foo", None).await?;
        let project_bar_uuid = storage_service.create_project("bar", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_projects),
        )
        .await;
        let req = test::TestRequest::get().uri("/v1/projects").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

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

        let storage_service = test_storage_service().await?;

        let _project_foo_uuid = storage_service.create_project("foo", None).await?;
        let project_bar_uuid = storage_service.create_project("bar", None).await?;

        let fake_extractor = FakeRoleExtractor::new(vec![YakManRoleBinding::ProjectRoleBinding(
            YakManUserProjectRole {
                project_uuid: project_bar_uuid.clone(),
                role: YakManRole::Admin,
            },
        )]);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_extractor))
                .service(get_projects),
        )
        .await;
        let req = test::TestRequest::get().uri("/v1/projects").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

        assert_eq!(1, value.as_array().unwrap().len());

        let first = &value.as_array().unwrap()[0];
        assert_eq!("bar", first["name"]);
        assert_eq!(project_bar_uuid, first["uuid"]);

        Ok(())
    }

    #[actix_web::test]
    async fn get_project_should_return_project() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let project_foo_uuid = storage_service.create_project("foo", None).await?;
        let _project_bar_uuid = storage_service.create_project("bar", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_project),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/projects/{project_foo_uuid}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

        assert_eq!("foo", value["name"]);
        assert_eq!(project_foo_uuid, value["uuid"]);

        Ok(())
    }

    #[actix_web::test]
    async fn get_project_should_not_return_projects_that_user_does_not_have() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let project_foo_uuid = storage_service.create_project("foo", None).await?;
        let project_bar_uuid = storage_service.create_project("bar", None).await?;

        let fake_extractor = FakeRoleExtractor::new(vec![YakManRoleBinding::ProjectRoleBinding(
            YakManUserProjectRole {
                project_uuid: project_bar_uuid.clone(),
                role: YakManRole::Admin,
            },
        )]);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_extractor))
                .service(get_project),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/projects/{project_foo_uuid}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 403);
        Ok(())
    }

    #[actix_web::test]
    async fn create_project_should_create_project_if_request_is_valid() -> Result<()> {
        prepare_for_actix_test()?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_storage_service().await?))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_project),
        )
        .await;
        let req = test::TestRequest::put()
            .uri("/v1/projects")
            .set_json(CreateProjectPayload {
                project_name: "valid-project-name".to_string(),
                notification_settings: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        Ok(())
    }

    #[actix_web::test]
    async fn create_project_should_return_bad_request_if_project_name_is_invalid() -> Result<()> {
        prepare_for_actix_test()?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_storage_service().await?))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_project),
        )
        .await;
        let req = test::TestRequest::put()
            .uri("/v1/projects")
            .set_json(CreateProjectPayload {
                project_name: "this is not a valid name".to_string(),
                notification_settings: None,
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

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_storage_service().await?))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_project),
        )
        .await;
        let req = test::TestRequest::put()
            .uri("/v1/projects")
            .set_json(CreateProjectPayload {
                project_name: "".to_string(),
                notification_settings: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status().as_u16();
        assert_eq!(400, status);
        Ok(())
    }
}
