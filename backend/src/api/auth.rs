use crate::{
    error::{ResetPasswordError, YakManApiError},
    middleware::{roles::YakManRoleBinding, YakManPrinciple},
    model::{YakManPublicPasswordResetLink, YakManRole},
    StateManager,
};
use actix_web::{
    post,
    web::{self, Json},
    HttpResponse, Responder,
};
use actix_web_grants::permissions::AuthDetails;
pub use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePasswordResetLink {
    pub user_uuid: String,
}

/// Create a new password reset link
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/auth/create-reset-password-link")]
pub async fn create_password_reset_link(
    payload: Json<CreatePasswordResetLink>,
    state: web::Data<StateManager>,
    principle: YakManPrinciple,
    auth_details: AuthDetails<YakManRoleBinding>,
) -> Result<impl Responder, YakManApiError> {
    let target_user_uuid = &payload.user_uuid;

    let user_uuid = match principle.user_uuid {
        Some(user_id) => user_id,
        None => return Err(YakManApiError::unauthorized()),
    };

    if &user_uuid != target_user_uuid {
        if !YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.permissions) {
            return Err(YakManApiError::forbidden());
        }
    }

    match state
        .get_service()
        .create_password_reset_link(&user_uuid)
        .await
    {
        Ok(reset_link) => return Ok(web::Json(reset_link)),
        Err(err) => {
            log::error!("failed to create password reset link: {}", err);
            return Err(YakManApiError::server_error("internal server error"));
        }
    };
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PasswordResetPayload {
    pub reset_link: YakManPublicPasswordResetLink,
    pub password: String,
}

/// Setup new user after set password link
#[utoipa::path(responses((status = 200)))]
#[post("/auth/reset-password")]
pub async fn reset_password(
    payload: Json<PasswordResetPayload>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    state
        .get_service()
        .reset_password_with_link(payload.reset_link.clone(), &payload.password)
        .await?;
    return Ok(HttpResponse::Ok().finish());
}

impl From<ResetPasswordError> for YakManApiError {
    fn from(value: ResetPasswordError) -> Self {
        match value {
            ResetPasswordError::ResetLinkNotFound => {
                return YakManApiError::not_found("reset link not found")
            }
            ResetPasswordError::InvalidUser => {
                return YakManApiError::bad_request("Invalid user id")
            }
            ResetPasswordError::InvalidEmail => {
                return YakManApiError::bad_request("Invalid email")
            }
            ResetPasswordError::ResetLinkExpired => {
                return YakManApiError::bad_request("Reset link expired")
            }
            ResetPasswordError::PasswordValidationError { error } => {
                return YakManApiError::bad_request(&error.to_string())
            }
            ResetPasswordError::StorageError { message } => {
                log::error!("Failed to reset password: {}", message);
                return YakManApiError::server_error("storage error");
            }
        }
    }
}
