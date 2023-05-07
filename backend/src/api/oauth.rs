use std::collections::HashMap;

use crate::{
    auth::{
        oauth_service::{OAUTH_ACCESS_TOKEN_COOKIE_NAME, OAUTH_REFRESH_TOKEN_COOKIE_NAME},
        LoginError,
    },
    middleware::roles::YakManRoleBinding,
    StateManager, YakManError,
};
use actix_web::{
    cookie::{time::Duration, Cookie},
    get, post,
    web::{self, Json},
    HttpRequest, HttpResponse, Responder,
};
use actix_web_grants::permissions::AuthDetails;
use log::{error, warn};
use oauth2::TokenResponse;
use yak_man_core::model::{
    oauth::{OAuthExchangePayload, OAuthInitPayload},
    response::GetUserRolesResponse,
    YakManRole,
};

/// Begins the oauth login flow
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/oauth2/init")]
pub async fn oauth_init(
    payload: Json<OAuthInitPayload>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let service = state.get_oauth_service();
    let redirect_uri = service.init_oauth(payload.challenge.clone());
    HttpResponse::Ok().body(redirect_uri)
}

/// Exchange an oauth code for token to complete login flow
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/oauth2/exchange")]
pub async fn oauth_exchange(
    payload: Json<OAuthExchangePayload>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let service = state.get_oauth_service();
    let token_service = state.get_token_service();

    let (username, user, token_result) = match service
        .exchange_oauth_code(
            String::from(payload.code.to_string()),
            String::from(payload.verifier.secret()),
        )
        .await
    {
        Ok(result) => result,
        Err(e) => {
            return match e {
                LoginError::UserNotRegistered => {
                    HttpResponse::Forbidden().body("User not registered")
                }
                e => {
                    error!("Login error {e:?}");
                    HttpResponse::InternalServerError().body("Failed to validate user")
                }
            }
        }
    };

    println!("{:?}", token_result);

    let (access_token_jwt, expire_timestamp) =
        match token_service.create_acess_token_jwt(&username, &user) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to create token {e}");
                return HttpResponse::InternalServerError().body("Failed to create token");
            }
        };

    let mut response = HttpResponse::Ok();
    response.cookie(
        Cookie::build(OAUTH_ACCESS_TOKEN_COOKIE_NAME, access_token_jwt)
            .path("/")
            .http_only(true)
            .max_age(Duration::milliseconds(expire_timestamp))
            .finish(),
    );

    if let Some(refresh_token) = token_result.refresh_token() {
        let refresh_token = refresh_token.secret();

        let encrypted_refresh_token = token_service.encrypt_refresh_token(refresh_token);
        response.cookie(
            Cookie::build(OAUTH_REFRESH_TOKEN_COOKIE_NAME, encrypted_refresh_token)
                .path("/api/oauth2/refresh") // TODO: This is currently a bug and will only work running locally with Trunk. (/api is not a path in release build)
                .http_only(true)
                .max_age(Duration::days(365 * 10)) // TODO: make this dynamic
                .finish(),
        );
    } else {
        warn!("No refresh token found, skipping refresh token cookie")
    }

    response.body("")
}

/// Use refresh_token cookie to generate new access token
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/oauth2/refresh")]
pub async fn oauth_refresh(request: HttpRequest, state: web::Data<StateManager>) -> HttpResponse {
    let cookie = match request.cookie(OAUTH_REFRESH_TOKEN_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().body("no refresh_token cookie found"),
    };

    let oauth_service = state.get_oauth_service();
    let storage = state.get_service();
    let token_service = state.get_token_service();

    let refresh_token = cookie.value();
    let access_token = match oauth_service.refresh_token(refresh_token).await {
        Ok(token) => token,
        Err(e) => {
            error!("Could not refresh token {e}");
            return HttpResponse::InternalServerError().body("Could not refresh token");
        }
    };

    let username = match oauth_service.get_username(&access_token).await {
        Ok(username) => username,
        Err(e) => {
            error!("Could not find username {e}");
            return HttpResponse::InternalServerError().body("Could not find username");
        }
    };

    let user = match storage.get_user(&username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Forbidden().body("User not found");
        }
        Err(e) => {
            error!("Could not fetch user {e}");
            return HttpResponse::InternalServerError().body("Could not load user");
        }
    };

    let (access_token_jwt, expire_timestamp) =
        match token_service.create_acess_token_jwt(&username, &user) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to create token {e}");
                return HttpResponse::InternalServerError().body("Failed to create token");
            }
        };

    HttpResponse::Ok()
        .cookie(
            Cookie::build(OAUTH_ACCESS_TOKEN_COOKIE_NAME, access_token_jwt)
                .path("/")
                .http_only(true)
                .max_age(Duration::milliseconds(expire_timestamp))
                .finish(),
        )
        .body("")
}

/// Endpoint to check if a user is logged in and get user roles
#[utoipa::path(responses((status = 200, body = GetUserRolesResponse)))]
#[get("/oauth2/user-roles")]
pub async fn get_user_roles(
    details: AuthDetails<YakManRoleBinding>,
) -> actix_web::Result<impl Responder, YakManError> {

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

    return Ok(web::Json(GetUserRolesResponse {
        global_roles: global_roles,
        roles: roles,
    }));
}
