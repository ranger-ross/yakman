use crate::auth::token::TokenService;
use crate::auth::{oauth_service::OAuthService, LoginError};
use crate::services::StorageService;
use crate::{
    auth::token::YakManTokenService,
    error::{CreatePasswordResetLinkError, ResetPasswordError, YakManApiError},
    middleware::{roles::YakManRoleBinding, YakManPrinciple},
    model::{YakManPublicPasswordResetLink, YakManRole},
    services::password::verify_password,
};
use actix_web::get;
use actix_web::{
    post,
    web::{self, Json},
    HttpResponse, Responder,
};
use actix_web_grants::permissions::AuthDetails;
use log::error;
use oauth2::PkceCodeChallenge;
use oauth2::PkceCodeVerifier;
pub use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use utoipa::openapi::{Object, ObjectBuilder, SchemaType};
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
    storage_service: web::Data<Arc<dyn StorageService>>,
    token_service: web::Data<Arc<YakManTokenService>>,
) -> Result<impl Responder, YakManApiError> {
    let password = match storage_service
        .get_password_by_email(&payload.username)
        .await
    {
        Ok(Some(password)) => password,
        _ => return Err(YakManApiError::unauthorized()),
    };
    match verify_password(&payload.password, password) {
        Ok(true) => {}
        _ => return Err(YakManApiError::unauthorized()),
    };

    let user: crate::model::YakManUser =
        match storage_service.get_user_by_email(&payload.username).await {
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

/// Create a new password reset link (non-oauth)
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/auth/create-reset-password-link")]
pub async fn create_password_reset_link(
    payload: Json<CreatePasswordResetLink>,
    storage_service: web::Data<Arc<dyn StorageService>>,
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

    match storage_service.create_password_reset_link(&user_uuid).await {
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

/// Validates a password reset link is valid (non-oauth)
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/auth/validate-reset-password-link")]
pub async fn validate_password_reset_link(
    payload: Json<ValidatePasswordResetLink>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    match storage_service
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

/// Setup new user after set password link (non-oauth)
#[utoipa::path(responses((status = 200)))]
#[post("/auth/reset-password")]
pub async fn reset_password(
    payload: Json<PasswordResetPayload>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    storage_service
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

fn string_schema() -> Object {
    ObjectBuilder::new().schema_type(SchemaType::String).build()
}

fn pkce_code_challenge_schema() -> Object {
    ObjectBuilder::new()
        .property(
            "code_challenge",
            ObjectBuilder::new().schema_type(SchemaType::String).build(),
        )
        .property(
            "code_challenge_method",
            ObjectBuilder::new().schema_type(SchemaType::String).build(),
        )
        .build()
}

fn reject_request_if_oauth_is_disabled(
    oauth_service: &Arc<dyn OAuthService>,
) -> Result<(), YakManApiError> {
    if !oauth_service.enabled() {
        return Err(YakManApiError::bad_request(
            "OAuth is disabled for this instance",
        ));
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthInitPayload {
    #[schema(schema_with=pkce_code_challenge_schema)]
    pub challenge: PkceCodeChallenge,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthInitResponse {
    redirect_uri: String,
    csrf_token: String,
    nonce: String,
}

/// Begins the oauth login flow
#[utoipa::path(responses((status = 200, body = OAuthInitResponse)))]
#[post("/oauth2/init")]
pub async fn oauth_init(
    payload: Json<OAuthInitPayload>,
    oauth_service: web::Data<Arc<dyn OAuthService>>,
) -> Result<impl Responder, YakManApiError> {
    reject_request_if_oauth_is_disabled(&oauth_service)?;

    let (redirect_uri, csrf_token, nonce) = oauth_service.init_oauth(payload.challenge.clone());

    Ok(web::Json(OAuthInitResponse {
        redirect_uri: redirect_uri,
        csrf_token: csrf_token.secret().clone(),
        nonce: nonce.secret().clone(),
    }))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthExchangePayload {
    pub state: String,
    pub code: String,
    #[schema(schema_with=string_schema)]
    pub verifier: PkceCodeVerifier,
    pub nonce: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthExchangeResponse {
    pub access_token: String,
    pub access_token_expire_timestamp: i64,
    pub refresh_token: Option<String>,
}

/// Exchange an oauth code for token to complete login flow
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/oauth2/exchange")]
pub async fn oauth_exchange(
    payload: Json<OAuthExchangePayload>,
    token_service: web::Data<Arc<YakManTokenService>>,
    oauth_service: web::Data<Arc<dyn OAuthService>>,
) -> Result<impl Responder, YakManApiError> {
    reject_request_if_oauth_is_disabled(&oauth_service)?;

    let (username, user, refresh_token, _picture) = match oauth_service
        .exchange_oauth_code(
            String::from(payload.code.to_string()),
            String::from(payload.verifier.secret()),
            payload.nonce.clone(),
        )
        .await
    {
        Ok(result) => result,
        Err(e) => {
            return match e {
                LoginError::UserNotRegistered => {
                    Err(YakManApiError::forbidden().set_message("User not registered"))
                }
                e => {
                    error!("Login error {e:?}");
                    Err(YakManApiError::server_error("Failed to validate user"))
                }
            }
        }
    };

    let (access_token_jwt, expire_timestamp) =
        match token_service.create_acess_token_jwt(&username, &user) {
            Ok(data) => data,
            Err(e) => {
                log::error!("Failed to create token {e}");
                return Err(YakManApiError::server_error("Failed to create token"));
            }
        };

    Ok(web::Json(OAuthExchangeResponse {
        access_token: access_token_jwt,
        access_token_expire_timestamp: expire_timestamp,
        refresh_token: refresh_token.map(|t| token_service.encrypt_refresh_token(t.secret())),
    }))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthRefreshTokenPayload {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthRefreshTokenResponse {
    pub access_token: String,
    pub access_token_expire_timestamp: i64,
}

/// Use refresh_token to generate new access token
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/auth/refresh")]
pub async fn oauth_refresh(
    payload: Json<OAuthRefreshTokenPayload>,
    storage_service: web::Data<Arc<dyn StorageService>>,
    token_service: web::Data<Arc<YakManTokenService>>,
    oauth_service: web::Data<Arc<dyn OAuthService>>,
) -> Result<impl Responder, YakManApiError> {
    let encrypted_refresh_token = &payload.refresh_token;
    log::info!("{encrypted_refresh_token}");
    let refresh_token = match token_service.decrypt_refresh_token(encrypted_refresh_token) {
        Ok(refresh_token) => refresh_token,
        Err(_) => return Err(YakManApiError::unauthorized().set_message("refresh_token not valid")),
    };
    let (_access_token, username) = match oauth_service.refresh_token(&refresh_token).await {
        Ok(token) => token,
        Err(e) => {
            log::error!("Could not refresh token {e}");
            return Err(YakManApiError::unauthorized().set_message("Could not refresh token"));
        }
    };

    let user = match storage_service.get_user_by_email(&username).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(YakManApiError::forbidden().set_message("User not found")),
        Err(e) => {
            log::error!("Could not fetch user {e}");
            return Err(YakManApiError::server_error("Could not load user"));
        }
    };

    let (access_token_jwt, expire_timestamp) =
        match token_service.create_acess_token_jwt(&username, &user) {
            Ok(data) => data,
            Err(e) => {
                log::error!("Failed to create token {e}");
                return Err(YakManApiError::server_error("Failed to create token"));
            }
        };

    Ok(web::Json(OAuthRefreshTokenResponse {
        access_token: access_token_jwt,
        access_token_expire_timestamp: expire_timestamp,
    }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetUserInfoResponse {
    pub profile_picture: Option<String>,
    pub global_roles: Vec<YakManRole>,
    pub roles: HashMap<String, YakManRole>,
}

/// Endpoint to get the currently logged in user's metadata and roles
#[utoipa::path(responses((status = 200, body = GetUserInfoResponse)))]
#[get("/auth/user-info")]
pub async fn get_user_info(
    details: AuthDetails<YakManRoleBinding>,
    principle: YakManPrinciple,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> actix_web::Result<impl Responder, YakManApiError> {
    let global_roles: Vec<YakManRole> = details
        .permissions
        .iter()
        .filter_map(|p| match p {
            YakManRoleBinding::GlobalRoleBinding(role) => Some(role.to_owned()),
            _ => None,
        })
        .collect();

    let roles: HashMap<String, YakManRole> = details
        .permissions
        .into_iter()
        .filter_map(|p| match p {
            YakManRoleBinding::ProjectRoleBinding(role) => Some((role.project_uuid, role.role)),
            _ => None,
        })
        .collect();

    let mut profile_picture = None;

    if let Some(user_uuid) = principle.user_uuid {
        if let Some(user) = storage_service.get_user_details(&user_uuid).await? {
            profile_picture = user.profile_picture;
        }
    }

    return Ok(web::Json(GetUserInfoResponse {
        profile_picture: profile_picture,
        global_roles: global_roles,
        roles: roles,
    }));
}
