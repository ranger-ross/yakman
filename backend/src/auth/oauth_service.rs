use super::{LoginError, RefreshTokenError};
use crate::model::YakManUser;
use crate::services::StorageService;
use log::debug;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use openidconnect::core::{
    CoreClient, CoreIdTokenClaims, CoreIdTokenVerifier, CoreProviderMetadata, CoreResponseType,
};
use openidconnect::reqwest::async_http_client;
use openidconnect::{AuthenticationFlow, IssuerUrl, Nonce};
use std::borrow::Cow;
use std::env;
use std::sync::Arc;

use base64::{engine::general_purpose, Engine as _};

pub const OAUTH_ACCESS_TOKEN_COOKIE_NAME: &str = "access_token";
pub const OAUTH_REFRESH_TOKEN_COOKIE_NAME: &str = "refresh_token";
pub const OIDC_NONCE_COOKIE_NAME: &str = "oidc_nonce";

pub struct OauthService {
    pub storage: Arc<dyn StorageService>,
    client: CoreClient,
    scopes: Vec<Scope>,
}

// TODO: Convert this to random per request and integrate it into the auth flow
fn static_nonce() -> Nonce {
    let c: u8 = 1;
    let nonrandom_bytes: Vec<u8> = (0..16).map(|_| c).collect();
    Nonce::new(general_purpose::URL_SAFE_NO_PAD.encode(nonrandom_bytes))
}

impl OauthService {
    pub async fn new(storage: Arc<dyn StorageService>) -> OauthService {
        let provider_metadata =
            CoreProviderMetadata::discover_async(get_issuer_url(), async_http_client)
                .await
                .unwrap(); // TODO: Better error handling

        // TODO: Support creating CoreClient without fetching metadata

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            get_client_id(),
            Some(get_client_secret()),
        )
        .set_redirect_uri(get_redirect_url());

        let scopes = get_oauth_scopes()
            .into_iter()
            .map(|s| Scope::new(s))
            .collect();

        return OauthService {
            storage: storage,
            client: client,
            scopes: scopes,
        };
    }

    pub fn init_oauth(&self, challenge: PkceCodeChallenge) -> (String, Nonce) {
        let (auth_url, _csrf_token, nonce) = self
            .client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scopes(self.scopes.clone())
            .set_pkce_challenge(challenge)
            .url();

        return (String::from(auth_url.as_str()), nonce);
    }

    pub async fn exchange_oauth_code(
        &self,
        code: String,
        verifier: String,
        nonce: String,
    ) -> Result<(String, YakManUser, Option<RefreshToken>), LoginError> {
        let pkce_verifier = PkceCodeVerifier::new(verifier);

        let data = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .set_redirect_uri(Cow::Owned(get_redirect_url()))
            .request_async(async_http_client)
            .await
            .map_err(|_| LoginError::FailedToExchangeCode)?;

        let id_token_verifier: CoreIdTokenVerifier = self.client.id_token_verifier();
        let id_token_claims: &CoreIdTokenClaims = data
            .extra_fields()
            .id_token()
            .expect("Server did not return an ID token")
            .claims(&id_token_verifier, &Nonce::new(nonce))
            .unwrap(); // TODO: FIX LATER

        let username = id_token_claims
            .email()
            .ok_or_else(|| LoginError::FailedToParseUsername)?
            .as_str()
            .to_string();

        if let Some(yakman_user) = self
            .storage
            .get_user(&username)
            .await
            .map_err(|_| LoginError::FailedToCheckRegisteredUsers)?
        {
            return Ok((
                username,
                yakman_user,
                data.refresh_token().map(|v| v.clone()), // TODO: Can we do this without cloning?
            ));
        }

        return Err(LoginError::UserNotRegistered);
    }

    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<(String, String), RefreshTokenError> {
        debug!("Attempting to refresh token");
        let token = RefreshToken::new(refresh_token.to_string());
        let response = self
            .client
            .exchange_refresh_token(&token)
            .request_async(async_http_client)
            .await
            .map_err(|e| RefreshTokenError::FailedToRefreshToken(Box::new(e)))?;

        let access_token = response.access_token().secret();

        let id_token_verifier: CoreIdTokenVerifier = self.client.id_token_verifier();
        let id_token_claims: &CoreIdTokenClaims = &response
            .extra_fields()
            .id_token()
            .expect("Server did not return an ID token")
            .to_owned()
             // For refresh tokens, nonce is not needed so use a no-op verifier
            .into_claims(&id_token_verifier, no_op_nonce_verifier) 
            .unwrap(); // TODO: FIX LATER

        let username = id_token_claims
            .email()
            .ok_or_else(|| RefreshTokenError::FailedToParseUsername)?
            .as_str()
            .to_string();

        return Ok((String::from(access_token), username));
    }
}

fn no_op_nonce_verifier(_: Option<&Nonce>) -> Result<(), String> {
    Ok(())
}

fn get_issuer_url() -> IssuerUrl {
    IssuerUrl::new(
        env::var("YAKMAN_OAUTH_ISSUER_URL").expect("$YAKMAN_OAUTH_ISSUER_URL is not set"),
    )
    .expect("YAKMAN_OAUTH_ISSUER_URL is not a valid URL")
}

fn get_token_url() -> TokenUrl {
    TokenUrl::new(env::var("YAKMAN_OAUTH_TOKEN_URL").expect("$YAKMAN_OAUTH_TOKEN_URL is not set"))
        .expect("YAKMAN_OAUTH_TOKEN_URL is not a valid URL")
}

fn get_redirect_url() -> RedirectUrl {
    RedirectUrl::new(
        env::var("YAKMAN_OAUTH_REDIRECT_URL").expect("$YAKMAN_OAUTH_REDIRECT_URL is not set"),
    )
    .expect("YAKMAN_OAUTH_REDIRECT_URL is not a valid URL")
}

fn get_client_id() -> ClientId {
    ClientId::new(env::var("YAKMAN_OAUTH_CLIENT_ID").expect("$YAKMAN_OAUTH_CLIENT_ID is not set"))
}

fn get_client_secret() -> ClientSecret {
    ClientSecret::new(
        env::var("YAKMAN_OAUTH_CLIENT_SECRET").expect("$YAKMAN_OAUTH_CLIENT_SECRET is not set"),
    )
}

fn get_oauth_scopes() -> Vec<String> {
    let scopes = env::var("YAKMAN_OAUTH_SCOPES").expect("$YAKMAN_OAUTH_SCOPES is not set");
    return scopes.split(",").map(|s| s.to_string()).collect();
}
