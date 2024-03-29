use std::{collections::HashSet, sync::Arc};

use crate::{
    api::is_alphanumeric_kebab_case,
    error::{CreateProjectError, DeleteProjectError, UpdateProjectError, YakManApiError},
    middleware::roles::YakManRoleBinding,
    model::{
        request::{
            CreateProjectPayload, ProjectNotificationSettings, ProjectNotificationType,
            UpdateProjectPayload,
        },
        YakManProject, YakManRole,
    },
    services::StorageService,
    settings,
};

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
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
            YakManRoleBinding::ProjectRoleBinding(r) => Some(r.project_id.clone()),
        })
        .collect();

    let projects: Vec<YakManProject> = storage_service
        .get_projects()
        .await?
        .into_iter()
        .filter(|p| user_has_global_role || allowed_projects.contains(&p.id))
        .collect();

    return Ok(web::Json(projects));
}

/// Get project by id
#[utoipa::path(responses((status = 200, body = YakManProjectDetails)))]
#[get("/v1/projects/{id}")]
pub async fn get_project(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<String>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    if auth_details.authorities.len() == 0 {
        return Err(YakManApiError::forbidden());
    }

    let project_id: String = path.into_inner();
    let has_role = YakManRoleBinding::has_any_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
            YakManRole::Viewer,
        ],
        &project_id,
        &auth_details.authorities,
    );

    if !has_role {
        return Err(YakManApiError::forbidden());
    }

    let Some(details) = storage_service.get_project_details(&project_id).await? else {
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

    validate_project(&project_name, payload.notification_settings.as_ref())?;

    return match storage_service
        .create_project(&project_name, payload.notification_settings)
        .await
    {
        Ok(project_id) => Ok(HttpResponse::Ok().body(project_id)),
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

/// Update a project
#[utoipa::path(request_body = UpdateProjectPayload, responses((status = 200, body = (), content_type = [])))]
#[post("/v1/projects/{id}")]
async fn update_project(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: web::Json<UpdateProjectPayload>,
    path: web::Path<String>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let payload = payload.into_inner();
    let project_name = payload.project_name.to_lowercase();

    let project_id: String = path.into_inner();
    let has_role = YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin],
        &project_id,
        &auth_details.authorities,
    );

    if !has_role {
        return Err(YakManApiError::forbidden());
    }

    validate_project(&project_name, payload.notification_settings.as_ref())?;

    return match storage_service
        .update_project(&project_id, &project_name, payload.notification_settings)
        .await
    {
        Ok(project_id) => Ok(HttpResponse::Ok().body(project_id)),
        Err(e) => match e {
            UpdateProjectError::StorageError { message } => {
                log::error!("Failed to create config {project_name}, error: {message}");
                Err(YakManApiError::server_error("Failed to create config"))
            }
            UpdateProjectError::DuplicateNameError { name: _ } => {
                Err(YakManApiError::bad_request("duplicate project"))
            }
            UpdateProjectError::ProjectNotFound => {
                Err(YakManApiError::bad_request("project not found"))
            }
        },
    };
}

/// Delete project by id
#[utoipa::path(responses((status = 200, body = ())))]
#[delete("/v1/projects/{id}")]
pub async fn delete_project(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<String>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    if auth_details.authorities.len() == 0 {
        return Err(YakManApiError::forbidden());
    }

    let project_id: String = path.into_inner();
    let has_role = YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin],
        &project_id,
        &auth_details.authorities,
    );

    if !has_role {
        return Err(YakManApiError::forbidden());
    }

    return match storage_service.delete_project(&project_id).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(DeleteProjectError::ProjectNotFound) => {
            Err(YakManApiError::not_found("project not found"))
        }
        Err(DeleteProjectError::StorageError { message }) => {
            log::error!("Failed to delete project {message}");
            Err(YakManApiError::server_error("failed to delete project"))
        }
    };
}

fn validate_project(
    project_name: &str,
    notification_settings: Option<&ProjectNotificationSettings>,
) -> Result<(), YakManApiError> {
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
    if let Some(notification) = &notification_settings {
        match &notification.notification_type {
            ProjectNotificationType::Slack { webhook_url } => validate_webhook_url(&webhook_url)?,
            ProjectNotificationType::Discord { webhook_url } => validate_webhook_url(&webhook_url)?,
        }
    }

    return Ok(());
}

fn validate_webhook_url(webhook_url: &str) -> Result<(), YakManApiError> {
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
    return Ok(());
}

#[cfg(test)]
mod tests {
    use core::panic;

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

        let project_foo_id = storage_service.create_project("foo", None).await?;
        let project_bar_id = storage_service.create_project("bar", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_projects),
        )
        .await;
        let req = test::TestRequest::get().uri("/v1/projects").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(200, resp.status().as_u16());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

        let first = &value.as_array().unwrap()[0];
        assert_eq!("foo", first["name"]);
        assert_eq!(project_foo_id, first["id"]);

        let second = &value.as_array().unwrap()[1];
        assert_eq!("bar", second["name"]);
        assert_eq!(project_bar_id, second["id"]);

        Ok(())
    }

    #[actix_web::test]
    async fn get_projects_should_not_return_projects_that_user_does_not_have() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let _project_foo_id = storage_service.create_project("foo", None).await?;
        let project_bar_id = storage_service.create_project("bar", None).await?;

        let fake_extractor = FakeRoleExtractor::new(vec![YakManRoleBinding::ProjectRoleBinding(
            YakManUserProjectRole {
                project_id: project_bar_id.clone(),
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
        assert_eq!(200, resp.status().as_u16());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

        assert_eq!(1, value.as_array().unwrap().len());

        let first = &value.as_array().unwrap()[0];
        assert_eq!("bar", first["name"]);
        assert_eq!(project_bar_id, first["id"]);

        Ok(())
    }

    #[actix_web::test]
    async fn get_project_should_return_project() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let project_foo_id = storage_service.create_project("foo", None).await?;
        let _project_bar_id = storage_service.create_project("bar", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_project),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/projects/{project_foo_id}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(200, resp.status().as_u16());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

        assert_eq!("foo", value["name"]);
        assert_eq!(project_foo_id, value["id"]);

        Ok(())
    }

    #[actix_web::test]
    async fn get_project_should_not_return_projects_that_user_does_not_have() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let project_foo_id = storage_service.create_project("foo", None).await?;
        let project_bar_id = storage_service.create_project("bar", None).await?;

        let fake_extractor = FakeRoleExtractor::new(vec![YakManRoleBinding::ProjectRoleBinding(
            YakManUserProjectRole {
                project_id: project_bar_id.clone(),
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
            .uri(&format!("/v1/projects/{project_foo_id}"))
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
        assert_eq!(200, resp.status().as_u16());
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

    #[actix_web::test]
    async fn update_project_should_update_project_if_request_is_valid() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let project_foo_id = storage_service.create_project("foo", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(update_project),
        )
        .await;
        let req = test::TestRequest::post()
            .uri(&format!("/v1/projects/{project_foo_id}"))
            .set_json(UpdateProjectPayload {
                project_name: "foo".to_string(),
                notification_settings: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(200, resp.status().as_u16());
        Ok(())
    }

    #[actix_web::test]
    async fn update_project_should_prevent_duplicate_project_names() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let project_foo_id = storage_service.create_project("foo", None).await?;
        let _project_bar_id = storage_service.create_project("bar", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(update_project),
        )
        .await;
        let req = test::TestRequest::post()
            .uri(&format!("/v1/projects/{project_foo_id}"))
            .set_json(UpdateProjectPayload {
                project_name: "bar".to_string(),
                notification_settings: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(400, resp.status().as_u16());
        Ok(())
    }

    #[actix_web::test]
    async fn update_project_should_respond_with_client_error_if_project_not_found() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let _project_foo_id = storage_service.create_project("foo", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(update_project),
        )
        .await;
        let req = test::TestRequest::post()
            .uri(&format!("/v1/projects/p48ad84e623f0")) // random id
            .set_json(UpdateProjectPayload {
                project_name: "foo".to_string(),
                notification_settings: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(400, resp.status().as_u16());
        Ok(())
    }

    #[actix_web::test]
    async fn update_project_should_valiate_project_name() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let project_foo_id = storage_service.create_project("foo", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(update_project),
        )
        .await;
        let req = test::TestRequest::post()
            .uri(&format!("/v1/projects/{project_foo_id}"))
            .set_json(UpdateProjectPayload {
                project_name: "invalid project".to_string(),
                notification_settings: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(400, resp.status().as_u16());
        Ok(())
    }

    #[actix_web::test]
    async fn update_project_should_check_permissions() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let project_foo_id = storage_service.create_project("foo", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::operator_role)) // Admin role needed
                .service(update_project),
        )
        .await;
        let req = test::TestRequest::post()
            .uri(&format!("/v1/projects/{project_foo_id}"))
            .set_json(UpdateProjectPayload {
                project_name: "foo".to_string(),
                notification_settings: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(403, resp.status().as_u16());
        Ok(())
    }

    #[actix_web::test]
    async fn delete_project_should_delete_project() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let project_foo_id = storage_service.create_project("foo", None).await?;
        let _project_bar_id = storage_service.create_project("bar", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(delete_project),
        )
        .await;
        let req = test::TestRequest::delete()
            .uri(&format!("/v1/projects/{project_foo_id}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(200, resp.status().as_u16());

        let project_details = storage_service.get_project_details(&project_foo_id).await?;
        if project_details.is_some() {
            panic!("project details was not delete");
        }

        let projects = storage_service.get_projects().await?;

        if let Some(_) = projects.iter().find(|p| p.id == project_foo_id) {
            panic!("project was not deleted")
        }

        Ok(())
    }

    #[actix_web::test]
    async fn delete_project_should_return_not_found_for_none_existent_project() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let _project_foo_id = storage_service.create_project("foo", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(delete_project),
        )
        .await;
        let req = test::TestRequest::delete()
            .uri(&format!("/v1/projects/p48ad84e623f0")) // fake id
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_project_should_check_permissions() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let project_foo_id = storage_service.create_project("foo", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::approver_role))
                .service(delete_project),
        )
        .await;
        let req = test::TestRequest::delete()
            .uri(&format!("/v1/projects/{project_foo_id}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(403, resp.status().as_u16());

        Ok(())
    }
}
