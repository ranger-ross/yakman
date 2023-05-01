use std::borrow::Cow;

use crate::StateManager;
use actix_web::{
    cookie::{time::Duration, Cookie},
    post,
    web::{self, Json},
    HttpResponse,
};
use log::info;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, EmptyExtraTokenFields, PkceCodeVerifier,
    RedirectUrl, StandardTokenResponse, TokenResponse, TokenUrl,
};
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

    let token_result = service.exchange_oauth_code(
        String::from(payload.code.to_string()),
        String::from(payload.verifier.secret()),
    )
    .await;

    println!("{:?}", token_result);

    HttpResponse::Ok()
        .cookie(
            Cookie::build("access_token", token_result.access_token().secret())
                .path("/")
                .http_only(true)
                .max_age(Duration::minutes(30)) // TODO: make this dynamic
                .finish(),
        )
        // .cookie(Cookie::build("refresh_token", token_result.refresh_token()).finish()) // TODO: Handle refresh token
        .body("")
}

