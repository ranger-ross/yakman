use crate::model::{
    request::{CreateConfigPayload, DeleteConfigPayload},
    YakManRole,
};
use crate::{
    api::is_alphanumeric_kebab_case,
    error::YakManApiError,
    error::{CreateConfigError, DeleteConfigError},
    middleware::roles::YakManRoleBinding,
    StateManager,
};
use actix_web::{delete, get, put, web, Responder};
use actix_web_grants::permissions::AuthDetails;
use log::error;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize)]
pub struct GetConfigsQuery {
    pub project: Option<String>,
}

/// List of all configs
#[utoipa::path(responses((status = 200, body = Vec<Config>)))]
#[get("/v1/configs")]
pub async fn get_configs(
    auth_details: AuthDetails<YakManRoleBinding>,
    query: web::Query<GetConfigsQuery>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let project_uuid = query.project.to_owned();
    let has_global_role = YakManRoleBinding::has_any_global_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
            YakManRole::Viewer,
        ],
        &auth_details.permissions,
    );

    if let Some(project_uuid) = &project_uuid {
        if !has_global_role
            && !YakManRoleBinding::has_any_role(
                vec![
                    YakManRole::Admin,
                    YakManRole::Approver,
                    YakManRole::Operator,
                    YakManRole::Viewer,
                ],
                project_uuid,
                &auth_details.permissions,
            )
        {
            return Err(YakManApiError::forbidden());
        }
    }

    let allowed_projects: HashSet<String> = auth_details
        .permissions
        .into_iter()
        .filter_map(|p| match p {
            YakManRoleBinding::ProjectRoleBinding(role) => Some(role.project_uuid),
            _ => None,
        })
        .collect();

    let service = state.get_service();
    let data = service.get_visible_configs(project_uuid).await?;

    if has_global_role {
        return Ok(web::Json(data));
    }

    let filtered_data = data
        .into_iter()
        .filter(|c| allowed_projects.contains(&c.project_uuid))
        .collect();

    return Ok(web::Json(filtered_data));
}

/// Create a new config
#[utoipa::path(request_body = CreateConfigPayload, responses((status = 200, body = String)))]
#[put("/v1/configs")]
async fn create_config(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: web::Json<CreateConfigPayload>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let payload = payload.into_inner();
    let config_name = payload.config_name.to_lowercase();
    let project_uuid = payload.project_uuid;

    if !YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &project_uuid,
        &auth_details.permissions,
    ) {
        return Err(YakManApiError::forbidden());
    }

    if config_name.is_empty() {
        return Err(YakManApiError::bad_request(
            "Invalid config name. Must not be empty",
        ));
    }

    if !is_alphanumeric_kebab_case(&config_name) {
        return Err(YakManApiError::bad_request(
            "Invalid config name. Must be alphanumeric kebab case",
        ));
    }

    let service = state.get_service();

    let projects = match service.get_projects().await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to load projects, error: {e:?}");
            return Err(YakManApiError::server_error("Failed to create config"));
        }
    };

    if projects
        .into_iter()
        .find(|p| p.uuid == project_uuid)
        .is_none()
    {
        return Err(YakManApiError::bad_request("Project does not exist"));
    }

    let result: Result<(), CreateConfigError> =
        service.create_config(&config_name, &project_uuid).await;

    return match result {
        Ok(()) => Ok(web::Json(config_name)),
        Err(e) => match e {
            CreateConfigError::StorageError { message } => {
                error!("Failed to create config {config_name}, error: {message}");
                Err(YakManApiError::server_error("Failed to create config"))
            }
            CreateConfigError::DuplicateConfigError { name: _ } => {
                Err(YakManApiError::bad_request("duplicate config"))
            }
        },
    };
}

/// Hide a config instance from the UI and API (data not deleted)
#[utoipa::path(request_body = DeleteConfigPayload, responses((status = 200, body = String)))]
#[delete("/v1/configs")]
async fn delete_config(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: web::Json<DeleteConfigPayload>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let payload = payload.into_inner();
    let config_name = payload.config_name.to_lowercase();
    let project_uuid = payload.project_uuid;

    if !YakManRoleBinding::has_any_role(
        vec![YakManRole::Admin],
        &project_uuid,
        &auth_details.permissions,
    ) {
        return Err(YakManApiError::forbidden());
    }

    if !is_alphanumeric_kebab_case(&config_name) {
        return Err(YakManApiError::bad_request(
            "Invalid config name. Must be alphanumeric kebab case",
        ));
    }

    let service = state.get_service();

    let result: Result<(), DeleteConfigError> = service.delete_config(&config_name).await;

    return match result {
        Ok(()) => Ok(web::Json(())),
        Err(e) => match e {
            DeleteConfigError::StorageError { message } => {
                error!("Failed to create config {config_name}, error: {message}");
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
        model::YakManUserProjectRole,
        test_utils::{fake_roles::FakeRoleExtractor, *},
    };
    use actix_web::{test, web::Data, App};
    use actix_web_grants::GrantsMiddleware;
    use anyhow::Result;
    use serde_json::Value;

    #[actix_web::test]
    async fn get_configs_should_return_configs() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        // Setup test project with 2 configs
        let project_uuid = state.service.create_project("test").await?;
        state
            .service
            .create_config("config1", &project_uuid)
            .await?;
        state
            .service
            .create_config("config2", &project_uuid)
            .await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_configs),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/configs?project={project_uuid}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp).await?;

        let arr = value.as_array().unwrap();

        assert_eq!(2, arr.len());

        let first = &arr[0];
        assert_eq!("config1", first["name"]);
        assert_eq!(false, first["hidden"]);
        assert_eq!(project_uuid.as_str(), first["project_uuid"]);

        let second = &arr[1];
        assert_eq!("config2", second["name"]);
        assert_eq!(false, second["hidden"]);
        assert_eq!(project_uuid.as_str(), second["project_uuid"]);

        Ok(())
    }

    #[actix_web::test]
    async fn get_configs_should_not_return_configs_for_other_projects() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        // Setup test 2 project with 1 config each
        let project1_uuid = state.service.create_project("proj1").await?;
        state
            .service
            .create_config("config1", &project1_uuid)
            .await?;
        let project2_uuid = state.service.create_project("proj2").await?;
        state
            .service
            .create_config("config2", &project2_uuid)
            .await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_configs),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/configs?project={project1_uuid}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp).await?;

        let arr = value.as_array().unwrap();

        assert_eq!(1, arr.len());

        let first = &arr[0];
        assert_eq!("config1", first["name"]);
        assert_eq!(false, first["hidden"]);
        assert_eq!(project1_uuid.as_str(), first["project_uuid"]);

        Ok(())
    }

    #[actix_web::test]
    async fn get_configs_should_not_return_forbidden_if_user_does_not_have_access_to_project(
    ) -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        // Setup test project with config
        let project1_uuid = state.service.create_project("proj1").await?;
        state
            .service
            .create_config("config1", &project1_uuid)
            .await?;

        let fake_role_extractor =
            FakeRoleExtractor::new(vec![YakManRoleBinding::ProjectRoleBinding(
                YakManUserProjectRole {
                    project_uuid: "other".to_string(), // fake, just some other project
                    role: YakManRole::Operator,
                },
            )]);

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state))
                .wrap(GrantsMiddleware::with_extractor(fake_role_extractor))
                .service(get_configs),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/configs?project={project1_uuid}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(403, resp.status().as_u16());

        Ok(())
    }


    #[actix_web::test]
    async fn get_configs_should_not_show_hidden_configs() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        // Setup test project with 2 configs
        let project_uuid = state.service.create_project("test").await?;
        state
            .service
            .create_config("config1", &project_uuid)
            .await?;
        state
            .service
            .create_config("config2", &project_uuid)
            .await?;

        // Hide config2
        state.service.delete_config("config2").await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_configs),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/configs?project={project_uuid}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp).await?;

        let arr = value.as_array().unwrap();

        assert_eq!(1, arr.len());

        let first = &arr[0];
        assert_eq!("config1", first["name"]);
        assert_eq!(false, first["hidden"]);
        assert_eq!(project_uuid.as_str(), first["project_uuid"]);

        Ok(())
    }

    #[actix_web::test]
    async fn create_configs_should_create_config_propertly() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        // Setup test project
        let project_uuid = state.service.create_project("test").await?;
      
        let app = test::init_service(
            App::new()
                .app_data(Data::new(state.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_config),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/v1/configs"))
            .set_json(&CreateConfigPayload {
                config_name: "foo-bar".to_string(),
                project_uuid: project_uuid
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(200, resp.status());


        let config = state.service.get_config("foo-bar").await?;
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!("foo-bar", config.name);
        assert_eq!(false, config.hidden);

        Ok(())
    }


    #[actix_web::test]
    async fn create_configs_should_block_invalid_config_names() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        // Setup test project
        let project_uuid = state.service.create_project("test").await?;
      
        let app = test::init_service(
            App::new()
                .app_data(Data::new(state.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_config),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/v1/configs"))
            .set_json(&CreateConfigPayload {
                config_name: "this is an invalid config name".to_string(),
                project_uuid: project_uuid
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(400, resp.status());

        Ok(())
    }


    #[actix_web::test]
    async fn create_configs_should_block_blank_config_names() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        // Setup test project
        let project_uuid = state.service.create_project("test").await?;
      
        let app = test::init_service(
            App::new()
                .app_data(Data::new(state.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_config),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/v1/configs"))
            .set_json(&CreateConfigPayload {
                config_name: "".to_string(),
                project_uuid: project_uuid
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(400, resp.status());

        Ok(())
    }



    #[actix_web::test]
    async fn create_configs_should_block_duplicate_config_names() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        // Setup test project
        let project_uuid = state.service.create_project("test").await?;
        state.service.create_config("foo-bar", &project_uuid).await?;
      
        let app = test::init_service(
            App::new()
                .app_data(Data::new(state.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_config),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/v1/configs"))
            .set_json(&CreateConfigPayload {
                config_name: "foo-bar".to_string(),
                project_uuid: project_uuid
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(400, resp.status());

        Ok(())
    }


}
