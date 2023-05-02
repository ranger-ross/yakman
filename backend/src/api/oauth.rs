use crate::{auth::LoginError, StateManager};
use actix_web::{
    cookie::{time::Duration, Cookie},
    post,
    web::{self, Json},
    HttpResponse,
};
use log::{error, warn};
use oauth2::TokenResponse;
use yak_man_core::model::{OAuthExchangePayload, OAuthInitPayload};

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

    let (username, role, token_result) = match service
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
        match token_service.create_acess_token_jwt(&username, &role) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to create token {e}");
                return HttpResponse::InternalServerError().body("Failed to create token");
            }
        };

    let mut response = HttpResponse::Ok();
    response.cookie(
        Cookie::build("access_token", access_token_jwt)
            .path("/")
            .http_only(true)
            .max_age(Duration::milliseconds(expire_timestamp))
            .finish(),
    );

    if let Some(refresh_token) = token_result.refresh_token() {
        let refresh_token = refresh_token.secret();

        let encrypted_refresh_token = token_service.encrypt_refresh_token(refresh_token);
        response.cookie(
            Cookie::build("resfresh_token", encrypted_refresh_token)
                .path("/")
                .http_only(true)
                .max_age(Duration::days(365 * 10)) // TODO: make this dynamic
                .finish(),
        );
    } else {
        warn!("No refresh token found, skipping refresh token cookie")
    }

    response.body("")
}
