use super::{LoginError, RefreshTokenError};
use crate::model::YakManUser;
use crate::services::StorageService;
use anyhow::{Context, Result};
use log::debug;
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use openidconnect::core::{
    CoreClient, CoreIdTokenClaims, CoreIdTokenVerifier, CoreProviderMetadata, CoreResponseType,
};
use openidconnect::reqwest::async_http_client;
use openidconnect::{AuthenticationFlow, IssuerUrl, Nonce};
use std::borrow::Cow;
use std::env;
use std::sync::Arc;

pub struct OauthService {
    pub storage: Arc<dyn StorageService>,
    client: CoreClient,
    scopes: Vec<Scope>,
    redirect_url: RedirectUrl,
}

impl OauthService {
    pub async fn new(storage: Arc<dyn StorageService>) -> Result<OauthService> {
        let provider_metadata =
            CoreProviderMetadata::discover_async(get_issuer_url()?, async_http_client).await?;

        // TODO: Support creating CoreClient without fetching metadata

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            get_client_id()?,
            Some(get_client_secret()?),
        )
        .set_redirect_uri(get_redirect_url()?);

        let scopes = get_oauth_scopes()
            .into_iter()
            .map(|s| Scope::new(s))
            .collect();

        return Ok(OauthService {
            storage: storage,
            client: client,
            scopes: scopes,
            redirect_url: get_redirect_url()?,
        });
    }

    pub fn init_oauth(&self, challenge: PkceCodeChallenge) -> (String, CsrfToken, Nonce) {
        let (auth_url, csrf_token, nonce) = self
            .client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scopes(self.scopes.clone())
            .set_pkce_challenge(challenge)
            .url();

        return (String::from(auth_url.as_str()), csrf_token, nonce);
    }

    pub async fn exchange_oauth_code(
        &self,
        code: String,
        verifier: String,
        nonce: String,
    ) -> Result<(String, YakManUser, Option<RefreshToken>, Option<String>), LoginError> {
        let pkce_verifier = PkceCodeVerifier::new(verifier);

        let data = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .set_redirect_uri(Cow::Owned(self.redirect_url.clone()))
            .request_async(async_http_client)
            .await
            .map_err(|_| LoginError::FailedToExchangeCode)?;

        let id_token_verifier: CoreIdTokenVerifier = self.client.id_token_verifier();
        let id_token_claims: &CoreIdTokenClaims = data
            .extra_fields()
            .id_token()
            .expect("Server did not return an ID token")
            .claims(&id_token_verifier, &Nonce::new(nonce))
            .map_err(|_| LoginError::FailedToParseClaims)?;

        let picture: Option<String> = id_token_claims
            .picture()
            .map(|p| p.get(None).map(|p| p.as_str().to_string()))
            .flatten();

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


            // log::info!("{id_token_claims:#?} {picture:?}");
            // Update the user's profile picture
            if let Some(profile_picture) = &picture {
                if let Ok(users) = self.storage.get_users().await {
                    let users: Vec<_> = users.into_iter().map(|mut user| {
                        if user.uuid == yakman_user.uuid {
                            user.profile_picture = Some(profile_picture.to_owned());
                        }
                        user 
                    })
                    .collect();
                    // Ignore the error, if the profile picture does not get update, 
                    // its fine just ignore and move on
                    let _ = self.storage.save_users(users).await;
                }
            }

            return Ok((
                username,
                yakman_user,
                data.refresh_token().clone().map(|v| v.clone()),
                picture,
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
            .map_err(|_| RefreshTokenError::FailedToParseClaims)?;

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

fn get_issuer_url() -> Result<IssuerUrl> {
    Ok(IssuerUrl::new(
        env::var("YAKMAN_OAUTH_ISSUER_URL")
            .with_context(|| "$YAKMAN_OAUTH_ISSUER_URL is not set")?,
    )
    .with_context(|| "YAKMAN_OAUTH_ISSUER_URL is not a valid URL")?)
}

fn get_token_url() -> Result<TokenUrl> {
    Ok(TokenUrl::new(
        env::var("YAKMAN_OAUTH_TOKEN_URL").with_context(|| "$YAKMAN_OAUTH_TOKEN_URL is not set")?,
    )
    .with_context(|| "YAKMAN_OAUTH_TOKEN_URL is not a valid URL")?)
}

fn get_redirect_url() -> Result<RedirectUrl> {
    Ok(RedirectUrl::new(
        env::var("YAKMAN_OAUTH_REDIRECT_URL")
            .with_context(|| "$YAKMAN_OAUTH_REDIRECT_URL is not set")?,
    )
    .with_context(|| "YAKMAN_OAUTH_REDIRECT_URL is not a valid URL")?)
}

fn get_client_id() -> Result<ClientId> {
    Ok(ClientId::new(
        env::var("YAKMAN_OAUTH_CLIENT_ID").with_context(|| "$YAKMAN_OAUTH_CLIENT_ID is not set")?,
    ))
}

fn get_client_secret() -> Result<ClientSecret> {
    Ok(ClientSecret::new(
        env::var("YAKMAN_OAUTH_CLIENT_SECRET")
            .with_context(|| "$YAKMAN_OAUTH_CLIENT_SECRET is not set")?,
    ))
}

fn get_oauth_scopes() -> Vec<String> {
    let scopes = env::var("YAKMAN_OAUTH_SCOPES").expect("$YAKMAN_OAUTH_SCOPES is not set");
    return scopes.split(",").map(|s| s.to_string()).collect();
}
