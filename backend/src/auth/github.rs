use super::{OAuthEmailResolver, OAuthEmailResolverError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct GitHubEmailResolver;

impl GitHubEmailResolver {
    pub fn new() -> GitHubEmailResolver {
        GitHubEmailResolver {}
    }
}

#[async_trait]
impl OAuthEmailResolver for GitHubEmailResolver {
    async fn resolve_email(&self, access_token: &str) -> Result<String, OAuthEmailResolverError> {
        let client = Client::builder()
            .user_agent("YakMan Backend")
            .build()
            .map_err(|e| OAuthEmailResolverError::FailedToBuildRequest(Box::new(e)))?;

        let response = client
            .get("https://api.github.com/user/emails")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| OAuthEmailResolverError::FailedToFetchUserData(Box::new(e)))?;

        let response_data = response
            .json::<Vec<GitHubResponse>>()
            .await
            .map_err(|e| OAuthEmailResolverError::FailedToExtractEmail(Box::new(e)))?;

        if let Some(email) = response_data
            .into_iter()
            .find(|e| e.primary)
            .map(|e| e.email)
        {
            Ok(email)
        } else {
            Err(OAuthEmailResolverError::EmailNotFound)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubResponse {
    email: String,
    primary: bool,
}
