use std::borrow::Cow;

use log::info;
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl,
};
use oauth2::{AuthorizationCode, EmptyExtraTokenFields, PkceCodeVerifier, StandardTokenResponse};

pub struct OauthService;

impl OauthService {
    pub fn init_oauth(&self, challenge: PkceCodeChallenge) -> String {
        info!("init oauth");
        let auth_url = get_auth_url();
        let token_url = get_token_url();

        // Create an OAuth2 client by specifying the client ID, client secret, authorization URL and
        // token URL.
        let client = BasicClient::new(
            get_client_id(),
            Some(get_client_secret()),
            auth_url,
            Some(token_url),
        )
        // Set the URL the user will be redirected to after the authorization process.
        .set_redirect_uri(get_redirect_url());

        // Generate the full authorization URL.
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            // Set the desired scopes.
            .add_scope(Scope::new("email".to_string()))
            // Set the PKCE code challenge.
            .set_pkce_challenge(challenge)
            .url();

        // This is the URL you should redirect the user to, in order to trigger the authorization
        // process.
        println!("Browse to: {}", auth_url);

        return String::from(auth_url.as_str());
    }

    pub async fn exchange_oauth_code(
        &self,
        code: String,
        verifier: String,
    ) -> StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> {
        let pkce_verifier = PkceCodeVerifier::new(verifier);

        info!("Using secret = {:?}", pkce_verifier.secret());

        let auth_url = get_auth_url();
        let token_url = get_token_url();

        // Create an OAuth2 client by specifying the client ID, client secret, authorization URL and
        // token URL.
        let client = BasicClient::new(
            get_client_id(),
            Some(get_client_secret()),
            auth_url,
            Some(token_url),
        );

        let redirect_url: RedirectUrl = get_redirect_url();

        let x = Cow::Owned(redirect_url);

        // Now you can trade it for an access token.
        return client
            .exchange_code(AuthorizationCode::new(code))
            // Set the PKCE code verifier.
            .set_pkce_verifier(pkce_verifier)
            .set_redirect_uri(x)
            .request_async(async_http_client)
            .await
            .unwrap();
    }
}

fn get_auth_url() -> AuthUrl {
    AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL")
}

fn get_token_url() -> TokenUrl {
    TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL")
}

fn get_redirect_url() -> RedirectUrl {
    RedirectUrl::new("http://127.0.0.1:8080/oauth-callback".to_string()).unwrap()
}

fn get_client_id() -> ClientId {
    ClientId::new(
        "".to_string(),
    )
}

fn get_client_secret() -> ClientSecret {
    ClientSecret::new("".to_string())
}
