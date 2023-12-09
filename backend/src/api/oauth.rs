use std::collections::HashMap;

use crate::{
    auth::LoginError, error::YakManApiError, middleware::roles::YakManRoleBinding,
    model::YakManRole, StateManager,
};
use actix_web::{
    get, post,
    web::{self, Json},
    Responder,
};
use actix_web_grants::permissions::AuthDetails;
use log::error;
use oauth2::PkceCodeChallenge;
use oauth2::PkceCodeVerifier;
pub use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthInitPayload {
    pub challenge: PkceCodeChallenge,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthInitResponse {
    redirect_uri: String,
    csrf_token: String,
    nonce: String,
}

/// Begins the oauth login flow
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/oauth2/init")]
pub async fn oauth_init(
    payload: Json<OAuthInitPayload>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let service = state.get_oauth_service();
    let (redirect_uri, csrf_token, nonce) = service.init_oauth(payload.challenge.clone());

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
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let service = state.get_oauth_service();
    let token_service = state.get_token_service();

    let (username, user, refresh_token, picture) = match service
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
        refresh_token: refresh_token.map(|t| t.secret().clone()),
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

/// Use refresh_token cookie to generate new access token
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/oauth2/refresh")]
pub async fn oauth_refresh(
    payload: Json<OAuthRefreshTokenPayload>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let oauth_service = state.get_oauth_service();
    let storage = state.get_service();
    let token_service = state.get_token_service();

    let encrypted_refresh_token = &payload.refresh_token;
    let refresh_token = match token_service.decrypt_refresh_token(encrypted_refresh_token) {
        Ok(refresh_token) => refresh_token,
        Err(_) => {
            return Err(YakManApiError::unauthorized().set_message("no refresh_token not valid"))
        }
    };
    let (_access_token, username) = match oauth_service.refresh_token(&refresh_token).await {
        Ok(token) => token,
        Err(e) => {
            error!("Could not refresh token {e}");
            return Err(YakManApiError::unauthorized().set_message("Could not refresh token"));
        }
    };

    let user = match storage.get_user(&username).await {
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
                error!("Failed to create token {e}");
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
#[get("/oauth2/user-info")]
pub async fn get_user_info(
    details: AuthDetails<YakManRoleBinding>,
    state: web::Data<StateManager>,
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

    let _storage = state.get_service();
    // TODO: fetch real profile picture
    let profile_picture = "https://lh3.googleusercontent.com/a/ACg8ocKyuO8Cp4A0EJZM8FJ1HADi6iWCg1S-nyZkWi5dxb_YZdM=s96-c".to_string();

    return Ok(web::Json(GetUserInfoResponse {
        profile_picture: Some(profile_picture),
        global_roles: global_roles,
        roles: roles,
    }));
}
