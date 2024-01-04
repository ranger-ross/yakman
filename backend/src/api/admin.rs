use crate::auth::token::API_KEY_PREFIX;
use crate::error::YakManApiError;
use crate::middleware::YakManPrinciple;
use crate::model::YakManApiKey;
use crate::model::{request::CreateYakManUserPayload, YakManRole, YakManUser};
use crate::{middleware::roles::YakManRoleBinding, StateManager};
use actix_web::{delete, Responder};
use actix_web::{
    get, put,
    web::{self, Json},
};
use actix_web_grants::permissions::AuthDetails;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
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
pub async fn create_api_keys(
    auth_details: AuthDetails<YakManRoleBinding>,
    state: web::Data<StateManager>,
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

    let projects = state.get_service().get_projects().await?;
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

    state.service.save_api_key(ak).await?;

    return Ok(web::Json(CreateApiKeyResponse {
        api_key: new_api_key,
    }));
}

/// Revoke an API key
#[utoipa::path(responses((status = 200, body = String)))]
#[delete("/admin/v1/api-keys/{id}")]
pub async fn delete_api_key(
    auth_details: AuthDetails<YakManRoleBinding>,
    state: web::Data<StateManager>,
    path: web::Path<String>,
) -> Result<impl Responder, YakManApiError> {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.permissions);

    if !is_admin {
        return Err(YakManApiError::forbidden());
    }

    let id = path.into_inner();
    state.get_service().delete_api_key(&id).await?;

    return Ok(web::Json(()));
}

#[cfg(test)]
mod tests {

    // TODO: Create API test to make sure that api key hashes are not leaked
}
