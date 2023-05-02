use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use log::info;
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, IntrospectionUrl, PkceCodeChallenge, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use oauth2::{AuthorizationCode, EmptyExtraTokenFields, PkceCodeVerifier, StandardTokenResponse};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::services::StorageService;

use super::LoginError;

// use super::errors::LoginError;

pub struct OauthService {
    pub storage: Arc<dyn StorageService>,
    client: BasicClient,
    scopes: Vec<Scope>,
}

impl OauthService {
    pub fn new(storage: Arc<dyn StorageService>) -> OauthService {
        let client = BasicClient::new(
            get_client_id(),
            Some(get_client_secret()),
            get_auth_url(),
            Some(get_token_url()),
        )
        // Set the URL the user will be redirected to after the authorization process.
        .set_redirect_uri(get_redirect_url())
        .set_introspection_uri(
            IntrospectionUrl::new("https://www.googleapis.com/oauth2/v3/tokeninfo".to_string())
                .unwrap(),
        );

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

    pub fn init_oauth(&self, challenge: PkceCodeChallenge) -> String {
        let (auth_url, csrf_token) = self
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
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, LoginError> {
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
            .map_err(|e| LoginError::FailedToFetchUserData(e))?;

        if let None = self
            .storage
            .get_user(&username)
            .await
            .map_err(|_| LoginError::FailedToCheckRegisteredUsers)?
        {
            return Err(LoginError::UserNotRegistered);
        }

        return Ok(data);
    }

    pub async fn get_username(
        &self,
        access_token: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        return get_github_email(access_token).await; // TODO: Support other providers in the future
                                                     // return get_google_email(access_token).await; // TODO: Support other providers in the future
    }
}

async fn get_google_email(access_token: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://www.googleapis.com/oauth2/v3/tokeninfo?access_token={access_token}");

    let resp = reqwest::get(url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    let username = resp.get("email").unwrap().to_owned();
    Ok(username)
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubResponse {
    email: String,
    primary: bool,
}

async fn get_github_email(access_token: &str) -> Result<String, Box<dyn std::error::Error>> {
    info!("Sending request to github {access_token}");
    let resp = Client::builder()
        .user_agent("YakMan Backend")
        .build()?
        .get("https://api.github.com/user/emails")
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<Vec<GitHubResponse>>()
        .await?;

    info!("{resp:?}");

    let email = resp
        .into_iter()
        .find(|e| e.primary)
        .map(|e| e.email)
        .expect("Not valid email found");
    Ok(email)
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
