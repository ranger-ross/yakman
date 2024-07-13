use crate::{
    api::validation::validate_kebab_case,
    error::{CreateConfigError, DeleteConfigError, YakManApiError},
    middleware::roles::YakManRoleBinding,
    model::response::ConfigPayload,
};
use crate::{model::YakManRole, services::StorageService};
use actix_web::{
    delete, get, put,
    web::{self, Json},
    HttpResponse, Responder,
};
use actix_web_grants::authorities::AuthDetails;
use actix_web_validation::validator::Validated;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize)]
pub struct GetConfigsQuery {
    pub project: Option<String>,
}

/// List of all configs
#[utoipa::path(responses((status = 200, body = Vec<YakManConfig>)))]
#[get("/v1/configs")]
pub async fn get_configs(
    auth_details: AuthDetails<YakManRoleBinding>,
    query: web::Query<GetConfigsQuery>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let project_id = query.project.to_owned();
    let has_global_role = YakManRoleBinding::has_any_global_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
            YakManRole::Viewer,
        ],
        &auth_details.authorities,
    );

    if let Some(project_id) = &project_id {
        if !has_global_role
            && !YakManRoleBinding::has_any_role(
                vec![
                    YakManRole::Admin,
                    YakManRole::Approver,
                    YakManRole::Operator,
                    YakManRole::Viewer,
                ],
                project_id,
                &auth_details.authorities,
            )
        {
            return Err(YakManApiError::forbidden());
        }
    }

    let allowed_projects: HashSet<String> = auth_details
        .authorities
        .iter()
        .filter_map(|p| match p {
            YakManRoleBinding::ProjectRoleBinding(role) => Some(role.project_id.clone()),
            _ => None,
        })
        .collect();

    let data = storage_service.get_visible_configs(project_id).await?;

    if has_global_role {
        return Ok(web::Json(data));
    }

    let filtered_data = data
        .into_iter()
        .filter(|c| allowed_projects.contains(&c.project_id))
        .collect();

    return Ok(web::Json(filtered_data));
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema, Validate)]
pub struct CreateConfigPayload {
    #[validate(length(min = 1), custom(function = "validate_kebab_case"))]
    pub config_name: String,
    #[validate(length(min = 1))]
    pub project_id: String,
}

/// Create a new config
#[utoipa::path(request_body = CreateConfigPayload, responses((status = 200, body = String)))]
#[put("/v1/configs")]
async fn create_config(
    auth_details: AuthDetails<YakManRoleBinding>,
    Validated(Json(payload)): Validated<Json<CreateConfigPayload>>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let config_name = payload.config_name.to_lowercase();
    let project_id = payload.project_id;

    if !YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &project_id,
        &auth_details.authorities,
    ) {
        return Err(YakManApiError::forbidden());
    }

    let project = match storage_service.get_project_details(&project_id).await {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to load projects, error: {e:?}");
            return Err(YakManApiError::server_error("Failed to create config"));
        }
    };

    let Some(_) = project else {
        return Err(YakManApiError::bad_request("Project does not exist"));
    };

    let result: Result<String, CreateConfigError> = storage_service
        .create_config(&config_name, &project_id)
        .await;

    return match result {
        Ok(config_id) => Ok(web::Json(ConfigPayload { config_id })),
        Err(e) => match e {
            CreateConfigError::StorageError { message } => {
                log::error!("Failed to create config {config_name}, error: {message}");
                Err(YakManApiError::server_error("Failed to create config"))
            }
            CreateConfigError::DuplicateConfigError { name: _ } => {
                Err(YakManApiError::bad_request("duplicate config"))
            }
        },
    };
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema, Validate)]
pub struct DeleteConfigPayload {
    #[validate(length(min = 1))]
    pub config_id: String,
    #[validate(length(min = 1))]
    pub project_id: String,
}

/// Hide a config instance from the UI and API (data not deleted)
#[utoipa::path(request_body = DeleteConfigPayload, responses((status = 200, body = (), content_type = [])))]
#[delete("/v1/configs")]
async fn delete_config(
    auth_details: AuthDetails<YakManRoleBinding>,
    Validated(Json(payload)): Validated<Json<DeleteConfigPayload>>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let config_id = payload.config_id.to_lowercase();
    let project_id = payload.project_id;

    if !YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin],
        &project_id,
        &auth_details.authorities,
    ) {
        return Err(YakManApiError::forbidden());
    }

    let result: Result<(), DeleteConfigError> = storage_service.delete_config(&config_id).await;

    return match result {
        Ok(()) => Ok(HttpResponse::Ok().finish()),
        Err(e) => match e {
            DeleteConfigError::StorageError { message } => {
                log::error!("Failed to create config {config_id}, error: {message}");
                Err(YakManApiError::server_error("Failed to delete config"))
            }
            DeleteConfigError::ConfigDoesNotExistError => {
                Err(YakManApiError::bad_request("config does not exist"))
            }
        },
    };
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        model::YakManProjectRole,
        test_utils::{fake_roles::FakeRoleExtractor, *},
    };
    use actix_web::{test, web::Data, App};
    use actix_web_grants::GrantsMiddleware;
    use anyhow::Result;
    use serde_json::Value;

    #[actix_web::test]
    async fn get_configs_should_return_configs() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        // Setup test project with 2 configs
        let project_id = storage_service.create_project("test", None).await?;
        storage_service
            .create_config("config1", &project_id)
            .await?;
        storage_service
            .create_config("config2", &project_id)
            .await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_configs),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/configs?project={project_id}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

        let arr = value.as_array().unwrap();

        assert_eq!(2, arr.len());

        let first = &arr[0];
        assert_eq!("config1", first["name"]);
        assert_eq!(false, first["hidden"]);
        assert_eq!(project_id.as_str(), first["project_id"]);

        let second = &arr[1];
        assert_eq!("config2", second["name"]);
        assert_eq!(false, second["hidden"]);
        assert_eq!(project_id.as_str(), second["project_id"]);

        Ok(())
    }

    #[actix_web::test]
    async fn get_configs_should_not_return_configs_for_other_projects() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        // Setup test 2 project with 1 config each
        let project1_id = storage_service.create_project("proj1", None).await?;
        storage_service
            .create_config("config1", &project1_id)
            .await?;
        let project2_id = storage_service.create_project("proj2", None).await?;
        storage_service
            .create_config("config2", &project2_id)
            .await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_configs),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/configs?project={project1_id}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

        let arr = value.as_array().unwrap();

        assert_eq!(1, arr.len());

        let first = &arr[0];
        assert_eq!("config1", first["name"]);
        assert_eq!(false, first["hidden"]);
        assert_eq!(project1_id.as_str(), first["project_id"]);

        Ok(())
    }

    #[actix_web::test]
    async fn get_configs_should_not_return_forbidden_if_user_does_not_have_access_to_project(
    ) -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        // Setup test project with config
        let project1_id = storage_service.create_project("proj1", None).await?;
        storage_service
            .create_config("config1", &project1_id)
            .await?;

        let fake_role_extractor =
            FakeRoleExtractor::new(vec![YakManRoleBinding::ProjectRoleBinding(
                YakManProjectRole {
                    project_id: "other".to_string(), // fake, just some other project
                    role: YakManRole::Operator,
                },
            )]);

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_role_extractor))
                .service(get_configs),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/configs?project={project1_id}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(403, resp.status().as_u16());

        Ok(())
    }

    #[actix_web::test]
    async fn get_configs_should_not_show_hidden_configs() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        // Setup test project with 2 configs
        let project_id = storage_service.create_project("test", None).await?;
        storage_service
            .create_config("config1", &project_id)
            .await?;
        let config2_id = storage_service
            .create_config("config2", &project_id)
            .await?;

        // Hide config2
        storage_service.delete_config(&config2_id).await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_configs),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/configs?project={project_id}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

        let arr = value.as_array().unwrap();

        assert_eq!(1, arr.len());

        let first = &arr[0];
        assert_eq!("config1", first["name"]);
        assert_eq!(false, first["hidden"]);
        assert_eq!(project_id.as_str(), first["project_id"]);

        Ok(())
    }

    #[actix_web::test]
    async fn create_configs_should_create_config_propertly() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        // Setup test project
        let project_id = storage_service.create_project("test", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_config),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/v1/configs"))
            .set_json(&CreateConfigPayload {
                config_name: "foo-bar".to_string(),
                project_id: project_id,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(200, resp.status());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;
        let config_id = value["config_id"].as_str().unwrap();

        let config = storage_service.get_config(config_id).await?;
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!("foo-bar", config.name);
        assert_eq!(false, config.hidden);

        Ok(())
    }

    #[actix_web::test]
    async fn create_configs_should_block_invalid_config_names() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        // Setup test project
        let project_id = storage_service.create_project("test", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_config),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/v1/configs"))
            .set_json(&CreateConfigPayload {
                config_name: "this is an invalid config name".to_string(),
                project_id: project_id,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(400, resp.status());

        Ok(())
    }

    #[actix_web::test]
    async fn create_configs_should_block_blank_config_names() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        // Setup test project
        let project_id = storage_service.create_project("test", None).await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_config),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/v1/configs"))
            .set_json(&CreateConfigPayload {
                config_name: "".to_string(),
                project_id: project_id,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(400, resp.status());

        Ok(())
    }

    #[actix_web::test]
    async fn create_configs_should_block_duplicate_config_names() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        // Setup test project
        let project_id = storage_service.create_project("test", None).await?;
        storage_service
            .create_config("foo-bar", &project_id)
            .await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_config),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/v1/configs"))
            .set_json(&CreateConfigPayload {
                config_name: "foo-bar".to_string(),
                project_id: project_id,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(400, resp.status());

        Ok(())
    }
}
