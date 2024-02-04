use crate::{
    error::{CreatePasswordResetLinkError, ResetPasswordError, YakManApiError},
    middleware::{roles::YakManRoleBinding, YakManPrinciple},
    model::{YakManPublicPasswordResetLink, YakManRole},
    services::password::verify_password,
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
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub access_token_expire_timestamp: i64,
    pub refresh_token: Option<String>,
}

/// Login for non-oauth users
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/auth/login")]
pub async fn login(
    payload: web::Form<LoginRequest>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let token_service = state.get_token_service();
    let storage = state.get_service();

    let password = match storage.get_password_by_email(&payload.username).await {
        Ok(Some(password)) => password,
        _ => return Err(YakManApiError::unauthorized()),
    };
    match verify_password(&payload.password, password) {
        Ok(true) => {}
        _ => return Err(YakManApiError::unauthorized()),
    };

    let user: crate::model::YakManUser = match storage.get_user_by_email(&payload.username).await {
        Ok(Some(user)) => user,
        _ => return Err(YakManApiError::unauthorized()),
    };

    let (access_token_jwt, expire_timestamp) =
        match token_service.create_acess_token_jwt(&user.email, &user) {
            Ok(data) => data,
            Err(e) => {
                log::error!("Failed to create token {e}");
                return Err(YakManApiError::server_error("Failed to create token"));
            }
        };

    return Ok(web::Json(LoginResponse {
        access_token: access_token_jwt,
        access_token_expire_timestamp: expire_timestamp,
        refresh_token: None, // TODO: Support refresh token
    }));
}

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
        Err(CreatePasswordResetLinkError::InvalidUser) => {
            return Err(YakManApiError::bad_request("Invalid user"))
        }
        Err(CreatePasswordResetLinkError::StorageError { message }) => {
            log::error!("failed to create password reset link: {}", message);
            return Err(YakManApiError::server_error("internal server error"));
        }
    };
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ValidatePasswordResetLink {
    pub id: String,
    pub user_uuid: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValidatePasswordResetLinkResponse {
    pub valid: bool,
}

/// Validates a password reset link is valid
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/auth/validate-reset-password-link")]
pub async fn validate_password_reset_link(
    payload: Json<ValidatePasswordResetLink>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    match state
        .get_service()
        .validate_password_reset_link(&payload.id, &payload.user_uuid)
        .await
    {
        Ok(is_valid) => {
            return Ok(web::Json(ValidatePasswordResetLinkResponse {
                valid: is_valid,
            }))
        }
        Err(err) => {
            log::error!("Failed to validate password reset link: {}", err);
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
            ResetPasswordError::PasswordHashError { error } => {
                log::warn!("password could not be hashed {:?}", error);
                return YakManApiError::bad_request("Password invalid");
            }
            ResetPasswordError::StorageError { message } => {
                log::error!("Failed to reset password: {}", message);
                return YakManApiError::server_error("storage error");
            }
        }
    }
}
