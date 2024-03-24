use crate::{
    adapters::errors::GenericStorageError,
    api::is_alphanumeric_kebab_case,
    error::{CreateTeamError, DeleteTeamError, YakManApiError},
    middleware::roles::YakManRoleBinding,
    model::{request::CreateTeamPayload, response::ConfigPayload},
};
use crate::{model::YakManRole, services::StorageService};
use actix_web::{delete, get, put, web, HttpResponse, Responder};
use actix_web_grants::authorities::AuthDetails;
use std::sync::Arc;

/// Get teams
#[utoipa::path(request_body = Vec<YakManTeam>, responses((status = 200, body = String)))]
#[get("/v1/teams")]
async fn get_teams(
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    return match storage_service.get_teams().await {
        Ok(teams) => Ok(web::Json(teams)),
        Err(e) => match e {
            GenericStorageError {
                message,
                raw_message,
            } => {
                log::error!("Failed to get team, error: {message}, raw: {raw_message}");
                Err(YakManApiError::server_error("Failed to get teams"))
            }
        },
    };
}

/// Get team by id
#[utoipa::path(request_body = YakManTeamDetails, responses((status = 200, body = String)))]
#[get("/v1/teams/{id}")]
async fn get_team(
    path: web::Path<String>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let team_id = path.into_inner();
    return match storage_service.get_team_details(&team_id).await {
        Ok(Some(teams)) => Ok(web::Json(teams)),
        Ok(None) => Err(YakManApiError::not_found("team not found")),
        Err(e) => match e {
            GenericStorageError {
                message,
                raw_message,
            } => {
                log::error!("Failed to get team, error: {message}, raw: {raw_message}");
                Err(YakManApiError::server_error("Failed to get teams"))
            }
        },
    };
}

/// Create a new team
#[utoipa::path(request_body = CreateTeamPayload, responses((status = 200, body = String)))]
#[put("/v1/teams")]
async fn create_team(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: web::Json<CreateTeamPayload>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let payload = payload.into_inner();

    if !YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.authorities) {
        return Err(YakManApiError::forbidden());
    }

    if !is_alphanumeric_kebab_case(&payload.name) {
        return Err(YakManApiError::bad_request("invalid team name"));
    }

    return match storage_service.create_team(payload).await {
        Ok(config_id) => Ok(web::Json(ConfigPayload { config_id })),
        Err(e) => match e {
            CreateTeamError::DuplicateTeam => Err(YakManApiError::bad_request("duplicate team")),
            CreateTeamError::StorageError { message } => {
                log::error!("Failed to create team, error: {message}");
                Err(YakManApiError::server_error("Failed to create team"))
            }
        },
    };
}

/// Delete a team
#[utoipa::path(request_body = (), responses((status = 200, body = String)))]
#[delete("/v1/teams/{id}")]
async fn delete_team(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<String>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let team_id = path.into_inner();

    if !YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.authorities) {
        return Err(YakManApiError::forbidden());
    }

    return match storage_service.delete_team(&team_id).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => match e {
            DeleteTeamError::TeamNotFound => Err(YakManApiError::bad_request("team not found")),
            DeleteTeamError::StorageError { message } => {
                log::error!("Failed to create team, error: {message}");
                Err(YakManApiError::server_error("Failed to create team"))
            }
        },
    };
}

// #[cfg(test)]
// mod tests {

//     use super::*;
//     use crate::{
//         model::YakManProjectRole,
//         test_utils::{fake_roles::FakeRoleExtractor, *},
//     };
//     use actix_web::{test, web::Data, App};
//     use actix_web_grants::GrantsMiddleware;
//     use anyhow::Result;
//     use serde_json::Value;

//     #[actix_web::test]
//     async fn get_configs_should_return_configs() -> Result<()> {
//         prepare_for_actix_test()?;

//         let storage_service = test_storage_service().await?;

//         // Setup test project with 2 configs
//         let project_id = storage_service.create_project("test", None).await?;
//         storage_service
//             .create_config("config1", &project_id)
//             .await?;
//         storage_service
//             .create_config("config2", &project_id)
//             .await?;

//         let app = test::init_service(
//             App::new()
//                 .app_data(Data::new(storage_service))
//                 .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
//                 .service(get_configs),
//         )
//         .await;
//         let req = test::TestRequest::get()
//             .uri(&format!("/v1/configs?project={project_id}"))
//             .to_request();
//         let resp = test::call_service(&app, req).await;
//         assert!(resp.status().is_success());

//         let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

//         let arr = value.as_array().unwrap();

//         assert_eq!(2, arr.len());

//         let first = &arr[0];
//         assert_eq!("config1", first["name"]);
//         assert_eq!(false, first["hidden"]);
//         assert_eq!(project_id.as_str(), first["project_id"]);

//         let second = &arr[1];
//         assert_eq!("config2", second["name"]);
//         assert_eq!(false, second["hidden"]);
//         assert_eq!(project_id.as_str(), second["project_id"]);

//         Ok(())
//     }

//     #[actix_web::test]
//     async fn get_configs_should_not_return_configs_for_other_projects() -> Result<()> {
//         prepare_for_actix_test()?;

//         let storage_service = test_storage_service().await?;

//         // Setup test 2 project with 1 config each
//         let project1_id = storage_service.create_project("proj1", None).await?;
//         storage_service
//             .create_config("config1", &project1_id)
//             .await?;
//         let project2_id = storage_service.create_project("proj2", None).await?;
//         storage_service
//             .create_config("config2", &project2_id)
//             .await?;

//         let app = test::init_service(
//             App::new()
//                 .app_data(Data::new(storage_service))
//                 .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
//                 .service(get_configs),
//         )
//         .await;
//         let req = test::TestRequest::get()
//             .uri(&format!("/v1/configs?project={project1_id}"))
//             .to_request();
//         let resp = test::call_service(&app, req).await;
//         assert!(resp.status().is_success());

//         let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

//         let arr = value.as_array().unwrap();

//         assert_eq!(1, arr.len());

//         let first = &arr[0];
//         assert_eq!("config1", first["name"]);
//         assert_eq!(false, first["hidden"]);
//         assert_eq!(project1_id.as_str(), first["project_id"]);

//         Ok(())
//     }

//     #[actix_web::test]
//     async fn get_configs_should_not_return_forbidden_if_user_does_not_have_access_to_project(
//     ) -> Result<()> {
//         prepare_for_actix_test()?;

//         let storage_service = test_storage_service().await?;

//         // Setup test project with config
//         let project1_id = storage_service.create_project("proj1", None).await?;
//         storage_service
//             .create_config("config1", &project1_id)
//             .await?;

//         let fake_role_extractor =
//             FakeRoleExtractor::new(vec![YakManRoleBinding::ProjectRoleBinding(
//                 YakManProjectRole {
//                     project_id: "other".to_string(), // fake, just some other project
//                     role: YakManRole::Operator,
//                 },
//             )]);

//         let app = test::init_service(
//             App::new()
//                 .app_data(Data::new(storage_service))
//                 .wrap(GrantsMiddleware::with_extractor(fake_role_extractor))
//                 .service(get_configs),
//         )
//         .await;
//         let req = test::TestRequest::get()
//             .uri(&format!("/v1/configs?project={project1_id}"))
//             .to_request();
//         let resp = test::call_service(&app, req).await;
//         assert_eq!(403, resp.status().as_u16());

//         Ok(())
//     }

//     #[actix_web::test]
//     async fn get_configs_should_not_show_hidden_configs() -> Result<()> {
//         prepare_for_actix_test()?;

//         let storage_service = test_storage_service().await?;

//         // Setup test project with 2 configs
//         let project_id = storage_service.create_project("test", None).await?;
//         storage_service
//             .create_config("config1", &project_id)
//             .await?;
//         let config2_id = storage_service
//             .create_config("config2", &project_id)
//             .await?;

//         // Hide config2
//         storage_service.delete_config(&config2_id).await?;

//         let app = test::init_service(
//             App::new()
//                 .app_data(Data::new(storage_service))
//                 .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
//                 .service(get_configs),
//         )
//         .await;
//         let req = test::TestRequest::get()
//             .uri(&format!("/v1/configs?project={project_id}"))
//             .to_request();
//         let resp = test::call_service(&app, req).await;
//         assert!(resp.status().is_success());

//         let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

//         let arr = value.as_array().unwrap();

//         assert_eq!(1, arr.len());

//         let first = &arr[0];
//         assert_eq!("config1", first["name"]);
//         assert_eq!(false, first["hidden"]);
//         assert_eq!(project_id.as_str(), first["project_id"]);

//         Ok(())
//     }

//     #[actix_web::test]
//     async fn create_configs_should_create_config_propertly() -> Result<()> {
//         prepare_for_actix_test()?;

//         let storage_service = test_storage_service().await?;

//         // Setup test project
//         let project_id = storage_service.create_project("test", None).await?;

//         let app = test::init_service(
//             App::new()
//                 .app_data(Data::new(storage_service.clone()))
//                 .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
//                 .service(create_config),
//         )
//         .await;
//         let req = test::TestRequest::put()
//             .uri(&format!("/v1/configs"))
//             .set_json(&CreateConfigPayload {
//                 config_name: "foo-bar".to_string(),
//                 project_id: project_id,
//             })
//             .to_request();
//         let resp = test::call_service(&app, req).await;
//         assert_eq!(200, resp.status());

//         let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;
//         let config_id = value["config_id"].as_str().unwrap();

//         let config = storage_service.get_config(config_id).await?;
//         assert!(config.is_some());
//         let config = config.unwrap();
//         assert_eq!("foo-bar", config.name);
//         assert_eq!(false, config.hidden);

//         Ok(())
//     }

//     #[actix_web::test]
//     async fn create_configs_should_block_invalid_config_names() -> Result<()> {
//         prepare_for_actix_test()?;

//         let storage_service = test_storage_service().await?;

//         // Setup test project
//         let project_id = storage_service.create_project("test", None).await?;

//         let app = test::init_service(
//             App::new()
//                 .app_data(Data::new(storage_service.clone()))
//                 .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
//                 .service(create_config),
//         )
//         .await;
//         let req = test::TestRequest::put()
//             .uri(&format!("/v1/configs"))
//             .set_json(&CreateConfigPayload {
//                 config_name: "this is an invalid config name".to_string(),
//                 project_id: project_id,
//             })
//             .to_request();
//         let resp = test::call_service(&app, req).await;
//         assert_eq!(400, resp.status());

//         Ok(())
//     }

//     #[actix_web::test]
//     async fn create_configs_should_block_blank_config_names() -> Result<()> {
//         prepare_for_actix_test()?;

//         let storage_service = test_storage_service().await?;

//         // Setup test project
//         let project_id = storage_service.create_project("test", None).await?;

//         let app = test::init_service(
//             App::new()
//                 .app_data(Data::new(storage_service.clone()))
//                 .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
//                 .service(create_config),
//         )
//         .await;
//         let req = test::TestRequest::put()
//             .uri(&format!("/v1/configs"))
//             .set_json(&CreateConfigPayload {
//                 config_name: "".to_string(),
//                 project_id: project_id,
//             })
//             .to_request();
//         let resp = test::call_service(&app, req).await;
//         assert_eq!(400, resp.status());

//         Ok(())
//     }

//     #[actix_web::test]
//     async fn create_configs_should_block_duplicate_config_names() -> Result<()> {
//         prepare_for_actix_test()?;

//         let storage_service = test_storage_service().await?;

//         // Setup test project
//         let project_id = storage_service.create_project("test", None).await?;
//         storage_service
//             .create_config("foo-bar", &project_id)
//             .await?;

//         let app = test::init_service(
//             App::new()
//                 .app_data(Data::new(storage_service.clone()))
//                 .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
//                 .service(create_config),
//         )
//         .await;
//         let req = test::TestRequest::put()
//             .uri(&format!("/v1/configs"))
//             .set_json(&CreateConfigPayload {
//                 config_name: "foo-bar".to_string(),
//                 project_id: project_id,
//             })
//             .to_request();
//         let resp = test::call_service(&app, req).await;
//         assert_eq!(400, resp.status());

//         Ok(())
//     }
// }
