use std::sync::Arc;

use crate::auth::token::API_KEY_PREFIX;
use crate::error::YakManApiError;
use crate::middleware::roles::YakManRoleBinding;
use crate::middleware::YakManPrinciple;
use crate::model::YakManApiKey;
use crate::model::{request::CreateYakManUserPayload, YakManRole, YakManUser};
use crate::services::StorageService;
use actix_web::{delete, HttpResponse, Responder};
use actix_web::{
    get, put,
    web::{self, Json},
};
use actix_web_grants::permissions::AuthDetails;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::openapi::security::Http;
use utoipa::ToSchema;
use uuid::Uuid;

/// Gets users
#[utoipa::path(responses((status = 200, body = Vec<YakManUser>)))]
#[get("/admin/v1/users")]
pub async fn get_yakman_users(
    auth_details: AuthDetails<YakManRoleBinding>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.permissions);

    if !is_admin {
        return Err(YakManApiError::forbidden());
    }

    let users = storage_service.get_users().await?;
    return Ok(web::Json(users));
}

/// Create YakMan user
#[utoipa::path(request_body = YakManUser, responses((status = 200, body = String)))]
#[put("/admin/v1/users")]
pub async fn create_yakman_user(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: Json<CreateYakManUserPayload>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.permissions);

    if !is_admin {
        return Err(YakManApiError::forbidden());
    }

    let mut users = storage_service.get_users().await.unwrap();
    let user = payload.into_inner();

    users.push(YakManUser {
        email: user.email,
        uuid: Uuid::new_v4().to_string(),
        role: user.role,
    });

    storage_service.save_users(users).await.unwrap();

    Ok(web::Json(()))
}

/// Get Api Keys
#[utoipa::path(responses((status = 200, body = Vec<YakManUser>)))]
#[get("/admin/v1/api-keys")]
pub async fn get_api_keys(
    auth_details: AuthDetails<YakManRoleBinding>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.permissions);

    if !is_admin {
        return Err(YakManApiError::forbidden());
    }

    let mut api_keys = storage_service.get_api_keys().await?;

    // Avoid exposing the hash outside of the API
    api_keys = api_keys
        .into_iter()
        .map(|mut key| {
            key.hash = String::default();
            key
        })
        .collect();

    return Ok(web::Json(api_keys));
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct CreateApiKeyRequest {
    pub project_uuid: String,
    pub role: YakManRole,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct CreateApiKeyResponse {
    pub api_key: String,
}

/// Create an api key
#[utoipa::path(responses((status = 200, body = CreateApiKeyResponse)))]
#[put("/admin/v1/api-keys")]
pub async fn create_api_key(
    auth_details: AuthDetails<YakManRoleBinding>,
    storage_service: web::Data<Arc<dyn StorageService>>,
    principle: YakManPrinciple,
    request: web::Json<CreateApiKeyRequest>,
) -> Result<impl Responder, YakManApiError> {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.permissions);

    if !is_admin {
        return Err(YakManApiError::forbidden());
    }

    let user_uuid = match &principle.user_uuid {
        Some(uuid) => uuid,
        None => return Err(YakManApiError::forbidden()),
    };

    let projects = storage_service.get_projects().await?;
    if !projects
        .iter()
        .any(|p| p.uuid == request.project_uuid.to_string())
    {
        return Err(YakManApiError::bad_request("Invalid project"));
    }

    let now = Utc::now().timestamp_millis();
    let new_api_key = format!("{API_KEY_PREFIX}{}", Uuid::new_v4().to_string());

    let ak = YakManApiKey {
        id: format!("apikey-{}", Uuid::new_v4().to_string()),
        hash: sha256::digest(&new_api_key),
        project_uuid: request.project_uuid.to_string(),
        role: request.role.clone(),
        created_at: now,
        created_by_uuid: user_uuid.to_string(),
    };

    storage_service.save_api_key(ak).await?;

    return Ok(web::Json(CreateApiKeyResponse {
        api_key: new_api_key,
    }));
}

/// Revoke an API key
#[utoipa::path(responses((status = 200, body = ())))]
#[delete("/admin/v1/api-keys/{id}")]
pub async fn delete_api_key(
    auth_details: AuthDetails<YakManRoleBinding>,
    storage_service: web::Data<Arc<dyn StorageService>>,
    path: web::Path<String>,
) -> Result<impl Responder, YakManApiError> {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.permissions);

    if !is_admin {
        return Err(YakManApiError::forbidden());
    }

    let id = path.into_inner();
    storage_service.delete_api_key(&id).await?;

    return Ok(HttpResponse::Ok().finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use actix_web::dev::Service;
    use actix_web::{test, web::Data, App, HttpMessage};
    use actix_web_grants::GrantsMiddleware;
    use anyhow::Result;
    use serde_json::Value;

    fn fake_api_key() -> YakManApiKey {
        YakManApiKey {
            id: "apikey-d66a57c5-a425-4157-b790-13756084d0cf".to_string(),
            hash: "5fd924625f6ab16a19cc9807c7c506ae1813490e4ba675f843d5a10e0baacdb8".to_string(),
            project_uuid: "91d16380-9df0-41dc-8542-c2dcf3633e7b".to_string(),
            role: YakManRole::Viewer,
            created_at: 1704330312738,
            created_by_uuid: "c34e15d0-0697-47c1-b36a-7f3456c68f1d".to_string(),
        }
    }

    #[actix_web::test]
    async fn get_api_keys_should_return_correct_data_without_leaking_api_key_hashes() -> Result<()>
    {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let fake_api_key = fake_api_key();
        storage_service.save_api_key(fake_api_key.clone()).await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_api_keys),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/admin/v1/api-keys")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp).await?;

        let first = &value.as_array().unwrap()[0];
        assert_eq!("apikey-d66a57c5-a425-4157-b790-13756084d0cf", first["id"]);
        assert_eq!(
            "91d16380-9df0-41dc-8542-c2dcf3633e7b",
            first["project_uuid"]
        );
        assert_eq!("Viewer", first["role"]);
        assert_eq!(1704330312738, first["created_at"].as_i64().unwrap());
        assert_eq!(
            "c34e15d0-0697-47c1-b36a-7f3456c68f1d",
            first["created_by_uuid"]
        );

        // Make sure the hash is not leak in the response (regardless of the json field)
        let raw_response_body = value.to_string();
        assert!(!raw_response_body.contains(&fake_api_key.hash.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn create_api_keys_should_create_the_api_key_properly() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let project_uuid = storage_service.create_project("foo").await?;

        let api_keys = storage_service.get_api_keys().await?;
        assert_eq!(0, api_keys.len());

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .wrap_fn(|req, srv| {
                    req.extensions_mut().insert(YakManPrinciple {
                        user_uuid: Some("c34e15d0-0697-47c1-b36a-7f3456c68f1d".to_string()),
                    });

                    srv.call(req)
                })
                .service(create_api_key),
        )
        .await;
        let req = test::TestRequest::put()
            .uri("/admin/v1/api-keys")
            .set_json(&CreateApiKeyRequest {
                project_uuid: project_uuid.clone(),
                role: YakManRole::Viewer,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        log::error!("{:#?}", resp.status());
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp).await?;

        assert!(!value["api_key"].is_null());
        assert!(value["api_key"].is_string());

        let api_keys = storage_service.get_api_keys().await?;
        assert_eq!(1, api_keys.len());

        let api_key = &api_keys[0];
        assert_eq!(project_uuid, api_key.project_uuid);
        assert_eq!(YakManRole::Viewer, api_key.role);
        assert_eq!(
            "c34e15d0-0697-47c1-b36a-7f3456c68f1d",
            api_key.created_by_uuid
        );

        Ok(())
    }

    #[actix_web::test]
    async fn delete_api_keys_should_delete_the_api_key_properly() -> Result<()> {
        prepare_for_actix_test()?;

        let storage_service = test_storage_service().await?;

        let fake_api_key = fake_api_key();
        storage_service.save_api_key(fake_api_key.clone()).await?;

        // Validate the api key was saved so after we delete it, we can make sure the list count changes
        let api_keys = storage_service.get_api_keys().await?;
        assert_eq!(1, api_keys.len());

        let app = test::init_service(
            App::new()
                .app_data(Data::new(storage_service.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(delete_api_key),
        )
        .await;
        let req = test::TestRequest::delete()
            .uri(&format!("/admin/v1/api-keys/{}", fake_api_key.id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let api_keys = storage_service.get_api_keys().await?;
        assert_eq!(0, api_keys.len());

        Ok(())
    }
}
