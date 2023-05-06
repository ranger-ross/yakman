use super::oauth_provider::OAuthProvider;
use super::{LoginError, OAuthEmailResolverError, RefreshTokenError};
use crate::services::StorageService;
use log::debug;
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, RefreshToken,
    Scope, TokenResponse, TokenUrl,
};
use oauth2::{AuthorizationCode, EmptyExtraTokenFields, PkceCodeVerifier, StandardTokenResponse};
use std::borrow::Cow;
use std::env;
use std::sync::Arc;
use yak_man_core::model::YakManRole;

pub const OAUTH_ACCESS_TOKEN_COOKIE_NAME: &str = "access_token";
pub const OAUTH_REFRESH_TOKEN_COOKIE_NAME: &str = "refresh_token";

pub struct OauthService {
    pub storage: Arc<dyn StorageService>,
    client: BasicClient,
    scopes: Vec<Scope>,
    oauth_provider: OAuthProvider,
}

impl OauthService {
    pub fn new(storage: Arc<dyn StorageService>) -> OauthService {
        let client = BasicClient::new(
            get_client_id(),
            Some(get_client_secret()),
            get_auth_url(),
            Some(get_token_url()),
        )
        .set_redirect_uri(get_redirect_url());
        // TODO: For custom oauth impls, allow introspection

        let scopes = get_oauth_scopes()
            .into_iter()
            .map(|s| Scope::new(s))
            .collect();

        let oauth_provider = OAuthProvider::from_env().expect("Could not create oauth provider");

        return OauthService {
            storage: storage,
            client: client,
            scopes: scopes,
            oauth_provider: oauth_provider,
        };
    }

    pub fn init_oauth(&self, challenge: PkceCodeChallenge) -> String {
        let (auth_url, _csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(self.scopes.clone())
            .set_pkce_challenge(challenge)
            .url();

        return String::from(auth_url.as_str());
    }

    pub async fn exchange_oauth_code(
        &self,
        code: String,
        verifier: String,
    ) -> Result<
        (
            String,
            YakManRole,
            StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        ),
        LoginError,
    > {
        let pkce_verifier = PkceCodeVerifier::new(verifier);

        let data = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .set_redirect_uri(Cow::Owned(get_redirect_url()))
            .request_async(async_http_client)
            .await
            .map_err(|_| LoginError::FailedToExchangeCode)?;

        let token: String = data.access_token().secret().clone();
        let username = self
            .get_username(&token)
            .await
            .map_err(|e| LoginError::FailedToFetchUserData(Box::new(e)))?;

        if let Some(yakman_user) = self
            .storage
            .get_user(&username)
            .await
            .map_err(|_| LoginError::FailedToCheckRegisteredUsers)?
        {
            return Ok((username, yakman_user.role, data));
        }

        return Err(LoginError::UserNotRegistered);
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<String, RefreshTokenError> {
        debug!("Attempting to refresh token");
        let token = RefreshToken::new(refresh_token.to_string());
        let response = self
            .client
            .exchange_refresh_token(&token)
            .request_async(async_http_client)
            .await
            .map_err(|e| RefreshTokenError::FailedToRefreshToken(Box::new(e)))?;

        let access_token = response.access_token().secret();
        return Ok(String::from(access_token));
    }

    pub async fn get_username(
        &self,
        access_token: &str,
    ) -> Result<String, OAuthEmailResolverError> {
        return self
            .oauth_provider
            .get_email_resolver()
            .resolve_email(access_token)
            .await;
    }
}

fn get_auth_url() -> AuthUrl {
    AuthUrl::new(env::var("YAKMAN_OAUTH_AUTH_URL").expect("$YAKMAN_OAUTH_AUTH_URL is not set"))
        .expect("YAKMAN_OAUTH_AUTH_URL is not a valid URL")
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
