use crate::{
    adapters::errors::GenericStorageError,
    api::validation::is_alphanumeric_kebab_case,
    error::{CreateTeamError, DeleteTeamError, UpdateTeamError, YakManApiError},
    middleware::roles::YakManRoleBinding,
    model::request::{CreateTeamPayload, UpdateTeamPayload},
};
use crate::{model::YakManRole, services::StorageService};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use actix_web_grants::authorities::AuthDetails;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// Get teams
#[utoipa::path(request_body = Vec<YakManTeam>, responses((status = 200, body = String)))]
#[get("/v1/teams")]
async fn get_teams(
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    return match storage_service.get_teams().await {
        Ok(teams) => Ok(web::Json(teams)),
        Err(GenericStorageError {
            message,
            raw_message,
        }) => {
            log::error!("Failed to get team, error: {message}, raw: {raw_message}");
            Err(YakManApiError::server_error("Failed to get teams"))
        }
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
        Err(GenericStorageError {
            message,
            raw_message,
        }) => {
            log::error!("Failed to get team, error: {message}, raw: {raw_message}");
            Err(YakManApiError::server_error("Failed to get teams"))
        }
    };
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTeamResponse {
    team_id: String,
}

/// Create a new team
#[utoipa::path(request_body = CreateTeamPayload, responses((status = 200, body = CreateTeamResponse)))]
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
        Ok(team_id) => Ok(web::Json(CreateTeamResponse { team_id })),
        Err(e) => match e {
            CreateTeamError::DuplicateTeam => {
                Err(YakManApiError::bad_request("duplicate team name"))
            }
            CreateTeamError::StorageError { message } => {
                log::error!("Failed to create team, error: {message}");
                Err(YakManApiError::server_error("Failed to create team"))
            }
        },
    };
}

/// Create a new team
#[utoipa::path(request_body = UpdateTeamPayload, responses((status = 200, body = CreateTeamResponse)))]
#[post("/v1/teams/{id}")]
async fn update_team(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<String>,
    payload: web::Json<UpdateTeamPayload>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let team_id = path.into_inner();
    let payload = payload.into_inner();

    if !YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.authorities) {
        return Err(YakManApiError::forbidden());
    }

    if !is_alphanumeric_kebab_case(&payload.name) {
        return Err(YakManApiError::bad_request("invalid team name"));
    }

    return match storage_service.update_team(&team_id, payload).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => match e {
            UpdateTeamError::TeamNotFound => Err(YakManApiError::bad_request("team not found")),
            UpdateTeamError::DuplicateTeam => {
                Err(YakManApiError::bad_request("duplicate team name"))
            }
            UpdateTeamError::StorageError { message } => {
                log::error!("Failed to update team, error: {message}");
                Err(YakManApiError::server_error("Failed to update team"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::request::CreateYakManUserPayload, test_utils::*};
    use actix_web::{test, web::Data, App};
    use actix_web_grants::GrantsMiddleware;
    use anyhow::{Context, Result};
    use serde_json::Value;

    #[actix_web::test]
    async fn create_team_should_create_team() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_team),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/v1/teams"))
            .set_json(&CreateTeamPayload {
                name: "foo".to_string(),
                global_roles: vec![],
                roles: vec![],
                team_member_user_ids: vec![],
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(200, resp.status().as_u16());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

        let team_id = value["team_id"].as_str().unwrap();

        // Validate team details was created
        let team_details = storage_service.get_team_details(team_id).await?.unwrap();
        assert_eq!(team_id, team_details.id);
        assert_eq!("foo", team_details.name);

        // Validate team was added to global list
        let teams = storage_service.get_teams().await?;
        assert!(teams.iter().find(|t| t.id == team_id).is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn update_team_should_update_team() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let team_id = storage_service
            .create_team(CreateTeamPayload {
                name: "foo".to_string(),
                global_roles: vec![],
                roles: vec![],
                team_member_user_ids: vec![],
            })
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(update_team),
        )
        .await;
        let req = test::TestRequest::post()
            .uri(&format!("/v1/teams/{team_id}"))
            .set_json(&UpdateTeamPayload {
                name: "updated-team-name".to_string(),
                global_roles: vec![],
                roles: vec![],
                team_member_user_ids: vec![],
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(200, resp.status().as_u16());

        // Validate team details was updated
        let team_details = storage_service.get_team_details(&team_id).await?.unwrap();
        assert_eq!(team_id, team_details.id);
        assert_eq!("updated-team-name", team_details.name);

        // Validate team was added to global list
        let teams = storage_service.get_teams().await?;
        assert!(teams
            .iter()
            .find(|t| t.id == team_id && t.name == "updated-team-name")
            .is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn update_team_should_remove_users() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let user_id = storage_service
            .create_user(CreateYakManUserPayload {
                email: "test@null.com".to_string(),
                role: None,
            })
            .await?;

        let team_id = storage_service
            .create_team(CreateTeamPayload {
                name: "foo".to_string(),
                global_roles: vec![],
                roles: vec![],
                team_member_user_ids: vec![user_id.clone()],
            })
            .await?;

        // Before running API test check that the user was added to team properly
        let user_details = storage_service
            .get_user_details(&user_id)
            .await?
            .context("User was not saved properly")?;
        assert_eq!(user_details.team_ids.len(), 1);
        assert_eq!(user_details.team_ids[0], team_id);

        let team_details = storage_service
            .get_team_details(&team_id)
            .await?
            .context("Team details was not saved properly")?;
        assert_eq!(team_details.member_user_ids.len(), 1);
        assert_eq!(team_details.member_user_ids[0], user_id);

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(update_team),
        )
        .await;
        let req = test::TestRequest::post()
            .uri(&format!("/v1/teams/{team_id}"))
            .set_json(&UpdateTeamPayload {
                name: "foo".to_string(),
                global_roles: vec![],
                roles: vec![],
                team_member_user_ids: vec![],
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(200, resp.status().as_u16());

        // Validate team details was updated
        let team_details = storage_service
            .get_team_details(&team_id)
            .await?
            .context("Could not find team details")?;

        assert_eq!(team_id, team_details.id);
        assert_eq!(0, team_details.member_user_ids.len());

        // Validate user details was updated
        let user_details = storage_service
            .get_user_details(&user_id)
            .await?
            .context("User was not saved properly")?;
        assert_eq!(user_details.team_ids.len(), 0);

        Ok(())
    }

    #[actix_web::test]
    async fn get_team_should_return_team() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let team_id = storage_service
            .create_team(CreateTeamPayload {
                name: "foo".to_string(),
                global_roles: vec![],
                roles: vec![],
                team_member_user_ids: vec![],
            })
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_team),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/teams/{team_id}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(200, resp.status().as_u16());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;

        assert_eq!(team_id, value["id"].as_str().unwrap());
        assert_eq!("foo", value["name"].as_str().unwrap());
        assert_eq!(0, value["global_roles"].as_array().unwrap().len());
        assert_eq!(0, value["roles"].as_array().unwrap().len());
        assert_eq!(0, value["member_user_ids"].as_array().unwrap().len());

        Ok(())
    }

    #[actix_web::test]
    async fn get_teams_should_return_teams() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let team_id_1 = storage_service
            .create_team(CreateTeamPayload {
                name: "foo".to_string(),
                global_roles: vec![],
                roles: vec![],
                team_member_user_ids: vec![],
            })
            .await?;

        let team_id_2 = storage_service
            .create_team(CreateTeamPayload {
                name: "bar".to_string(),
                global_roles: vec![],
                roles: vec![],
                team_member_user_ids: vec![],
            })
            .await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_teams),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/v1/teams"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(200, resp.status().as_u16());

        let value: Value = body_to_json_value(resp.map_into_boxed_body()).await?;
        println!("{value:#?}");
        let value = value.as_array().context("response was not array")?;

        assert_eq!(2, value.len());

        assert_eq!(team_id_1, value[0]["id"]);
        assert_eq!("foo", value[0]["name"]);
        assert_eq!(team_id_2, value[1]["id"]);
        assert_eq!("bar", value[1]["name"]);

        Ok(())
    }
}
