use crate::error::YakManApiError;
use crate::model::YakManApiKey;
use crate::model::{request::CreateYakManUserPayload, YakManRole, YakManUser};
use crate::{middleware::roles::YakManRoleBinding, StateManager};
use actix_web::Responder;
use actix_web::{
    get, put,
    web::{self, Json},
};
use actix_web_grants::permissions::AuthDetails;
use chrono::Utc;
use uuid::Uuid;

/// Gets users
#[utoipa::path(responses((status = 200, body = Vec<YakManUser>)))]
#[get("/admin/v1/users")]
pub async fn get_yakman_users(
    auth_details: AuthDetails<YakManRoleBinding>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.permissions);

    if !is_admin {
        return Err(YakManApiError::forbidden());
    }

    let users = state.get_service().get_users().await?;
    return Ok(web::Json(users));
}

/// Create YakMan user
#[utoipa::path(request_body = YakManUser, responses((status = 200, body = String)))]
#[put("/admin/v1/users")]
pub async fn create_yakman_user(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: Json<CreateYakManUserPayload>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.permissions);

    if !is_admin {
        return Err(YakManApiError::forbidden());
    }

    let mut users = state.get_service().get_users().await.unwrap();
    let user = payload.into_inner();

    users.push(YakManUser {
        email: user.email,
        uuid: Uuid::new_v4().to_string(),
        role: user.role,
    });

    state.get_service().save_users(users).await.unwrap();

    Ok(web::Json(()))
}

/// Get Api Keys
#[utoipa::path(responses((status = 200, body = Vec<YakManUser>)))]
#[get("/admin/v1/api-keys")]
pub async fn get_api_keys(
    auth_details: AuthDetails<YakManRoleBinding>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.permissions);

    if !is_admin {
        return Err(YakManApiError::forbidden());
    }

    let mut api_keys = state.get_service().get_api_keys().await?;

    // let now = Utc::now().timestamp_millis();
    // let ak = YakManApiKey {
    //     id: Uuid::new_v4().to_string(),
    //     hash: "5fd924625f6ab16a19cc9807c7c506ae1813490e4ba675f843d5a10e0baacdb8".to_string(),
    //     project_uuid: "6a4ced76-f65c-43aa-9d33-5027c79bda71".to_string(),
    //     role: YakManRole::Viewer,
    //     created_at: now,
    //     created_by_uuid: "fda58896-e0ac-49e9-8a46-8973610db9ae".to_string(),
    // };

    // state.service.save_api_key(ak).await.unwrap();

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

// TODO: Create API test to make sure that api key hashes are not leaked
