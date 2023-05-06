use super::github::GitHubEmailResolver;
use super::google::GoogleEmailResolver;
use super::OAuthEmailResolver;
use std::env;
use thiserror::Error;

pub enum OAuthProvider {
    GitHub,
    Google,
}

#[derive(Error, Debug)]
pub enum OauthProviderCreateError {
    #[error("Failed to exchange token")]
    FailedToLoadEnvVar(Box<dyn std::error::Error>),
    #[error("Failed to exchange token")]
    UnknownProvider,
}

impl OAuthProvider {
    pub fn from_env() -> Result<OAuthProvider, OauthProviderCreateError> {
        let v = env::var("YAKMAN_OAUTH_PROVIDER")
            .map_err(|e| OauthProviderCreateError::FailedToLoadEnvVar(Box::new(e)))?;

        let provider = match v.as_str() {
            "GITHUB" => OAuthProvider::GitHub,
            "GOOGLE" => OAuthProvider::Google,
            _ => return Err(OauthProviderCreateError::UnknownProvider),
        };

        Ok(provider)
    }

    pub fn get_email_resolver(&self) -> Box<dyn OAuthEmailResolver> {
        return match self {
            OAuthProvider::GitHub => Box::new(GitHubEmailResolver::new()),
            OAuthProvider::Google => Box::new(GoogleEmailResolver::new()),
        };
    }
}
