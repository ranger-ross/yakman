use log::info;
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{
    reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Serialize;
use std::{env, sync::Arc};
use utoipa::OpenApi;

pub struct OauthService;

impl OauthService {
    pub fn init_oauth(&self) -> String {
        info!("init oauth");
        let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .expect("Invalid authorization endpoint URL");
        let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
            .expect("Invalid token endpoint URL");

        // Create an OAuth2 client by specifying the client ID, client secret, authorization URL and
        // token URL.
        let client = BasicClient::new(
            ClientId::new(
                ""
                    .to_string(),
            ),
            Some(ClientSecret::new(
                "".to_string(),
            )),
            auth_url,
            Some(token_url),
        )
        // Set the URL the user will be redirected to after the authorization process.
        .set_redirect_uri(RedirectUrl::new("http://127.0.0.1:8080".to_string()).unwrap());

        // Generate a PKCE challenge.

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the full authorization URL.
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            // Set the desired scopes.
            .add_scope(Scope::new("email".to_string()))
            // .add_scope(Scope::new("offline_access".to_string()))
            // Set the PKCE code challenge.
            .set_pkce_challenge(pkce_challenge)
            .url();

        // This is the URL you should redirect the user to, in order to trigger the authorization
        // process.
        println!("Browse to: {}", auth_url);

        return String::from(auth_url.as_str());
    }
}
