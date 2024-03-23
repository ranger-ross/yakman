use std::collections::HashMap;
use std::sync::Arc;

use crate::error::YakManApiError;
use crate::middleware::roles::YakManRoleBinding;
use crate::middleware::YakManPrinciple;
use crate::model::{request::CreateYakManUserPayload, YakManRole, YakManUser};
use crate::services::id::generate_user_id;
use crate::services::StorageService;
use actix_web::{
    get, put,
    web::{self, Json},
};
use actix_web::{HttpResponse, Responder};
use actix_web_grants::authorities::AuthDetails;
use serde::Serialize;
use utoipa::ToSchema;

/// Gets users
#[utoipa::path(responses((status = 200, body = Vec<YakManUser>)))]
#[get("/v1/users")]
pub async fn get_yakman_users(
    auth_details: AuthDetails<YakManRoleBinding>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.authorities);

    if !is_admin {
        return Err(YakManApiError::forbidden());
    }

    let users = storage_service.get_users().await?;
    return Ok(web::Json(users));
}

/// Create YakMan user
#[utoipa::path(request_body = YakManUser, responses((status = 200, body = (), content_type = [])))]
#[put("/v1/users")]
pub async fn create_yakman_user(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: Json<CreateYakManUserPayload>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.authorities);

    if !is_admin {
        return Err(YakManApiError::forbidden());
    }

    let mut users = storage_service.get_users().await.unwrap();
    let user = payload.into_inner();

    users.push(YakManUser {
        email: user.email,
        id: generate_user_id(),
        role: user.role,
    });

    storage_service.save_users(users).await.unwrap();

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetUserInfoResponse {
    pub profile_picture: Option<String>,
    pub global_roles: Vec<YakManRole>,
    pub roles: HashMap<String, YakManRole>,
}

/// Endpoint to get the currently logged in user's metadata and roles
#[utoipa::path(responses((status = 200, body = GetUserInfoResponse)))]
#[get("/v1/user-info")]
pub async fn get_user_info(
    details: AuthDetails<YakManRoleBinding>,
    principle: YakManPrinciple,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> actix_web::Result<impl Responder, YakManApiError> {
    let global_roles: Vec<YakManRole> = details
        .authorities
        .iter()
        .filter_map(|p| match p {
            YakManRoleBinding::GlobalRoleBinding(role) => Some(role.to_owned()),
            _ => None,
        })
        .collect();

    let roles: HashMap<String, YakManRole> = details
        .authorities
        .iter()
        .filter_map(|p| match p {
            YakManRoleBinding::ProjectRoleBinding(role) => {
                Some((role.project_id.clone(), role.role.clone()))
            }
            _ => None,
        })
        .collect();

    let mut profile_picture = None;

    if let Some(user_id) = principle.user_id {
        if let Some(user) = storage_service.get_user_details(&user_id).await? {
            profile_picture = user.profile_picture;
        }
    }

    return Ok(web::Json(GetUserInfoResponse {
        profile_picture: profile_picture,
        global_roles: global_roles,
        roles: roles,
    }));
}
